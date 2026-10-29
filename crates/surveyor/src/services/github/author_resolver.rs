//! Resolution of commit authors to GitHub users.

use crate::prelude::*;

/// Resolve commit authors to GitHub users.
pub struct AuthorResolver {
    commit_author: Arc<dyn GetCommitAuthor>,
    /// GitHub repo, or `None` if unknown.
    repo: Option<RepoName>,
    /// GitHub login of each author email looked up so far.
    logins: Mutex<HashMap<String, Option<String>>>,
}

impl AuthorResolver {
    /// Resolve the author of `commit` to a GitHub login.
    ///
    /// - Looks up each author email once and caches the login, or `None` if the lookup failed
    /// - Returns `None` if the GitHub repo is unknown
    /// - Logs lookup failures at trace instead of failing
    pub fn resolve(&self, commit: &Commit) -> Option<String> {
        let repo = self.repo.as_ref()?;
        let mut logins = self.logins.lock().expect("lock should not be poisoned");
        logins
            .entry(commit.author.email.clone())
            .or_insert_with(|| self.get_login(repo, commit))
            .clone()
    }

    /// Get the GitHub login of the author of `commit`.
    fn get_login(&self, repo: &RepoName, commit: &Commit) -> Option<String> {
        self.commit_author
            .get_commit_author(repo, &commit.hash)
            .inspect_err(|report| {
                trace!(
                    name = commit.author.name,
                    error = %report,
                    "Unable to get the GitHub login, so using the git author name"
                );
            })
            .ok()
    }
}

impl FromServices for AuthorResolver {
    type Error = ResolveError;

    fn from_services(services: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        let repo = services
            .get_trait::<dyn GetRepoName>()?
            .get_repo_name()
            .inspect_err(|report| {
                debug!(error = %report, "Unable to get the GitHub repo, so using git author names");
            })
            .ok();
        Ok(Self {
            repo,
            commit_author: services.get_trait::<dyn GetCommitAuthor>()?,
            logins: Mutex::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`AuthorResolver::resolve`] returns the login, looking up each email once.
    #[test]
    fn author_resolver_resolve() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitAuthorSpy {
                logins: HashMap::from([(CommitHash::fixture("aaa"), "ada".to_owned())]),
                ..GetCommitAuthorSpy::default()
            })
            .build();
        let resolver = services.expect::<AuthorResolver>();
        let commits = [
            Commit {
                hash: CommitHash::fixture("aaa"),
                ..Commit::fixture()
            },
            Commit {
                hash: CommitHash::fixture("bbb"),
                ..Commit::fixture()
            },
            Commit {
                hash: CommitHash::fixture("ccc"),
                author: Author {
                    name: "Bob".to_owned(),
                    email: "bob@example.com".to_owned(),
                    github: None,
                },
                ..Commit::fixture()
            },
        ];
        // Act
        let logins: Vec<Option<String>> = commits
            .iter()
            .map(|commit| resolver.resolve(commit))
            .collect();
        // Assert
        assert_eq!(
            logins,
            [Some("ada".to_owned()), Some("ada".to_owned()), None]
        );
        let hashes = services.expect::<GetCommitAuthorSpy>().get_hashes();
        assert_eq!(
            hashes,
            [CommitHash::fixture("aaa"), CommitHash::fixture("ccc")]
        );
    }

    /// [`AuthorResolver::resolve`] returns no login without a GitHub repo.
    #[test]
    fn author_resolver_resolve_no_repo() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetRepoNameStub {
                repo_name: Err(GetRepoNameError::Gh),
            })
            .build();
        let resolver = services.expect::<AuthorResolver>();
        // Act
        let login = resolver.resolve(&Commit::fixture());
        // Assert
        assert_eq!(login, None);
        let hashes = services.expect::<GetCommitAuthorSpy>().get_hashes();
        assert_eq!(hashes, Vec::new());
    }
}
