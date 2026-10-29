//! Retrieval of the parsed commits since a version.

use crate::prelude::*;

/// Get the commits since a version as parsed conventional commits.
#[derive(FromServices)]
pub struct CommitProvider {
    commits: Arc<dyn GetCommits>,
    parser: Arc<CommitParser>,
}

impl CommitProvider {
    /// Get the non-merge commits in `v<base>..HEAD`, newest first, with reverts marked.
    ///
    /// - Gets every commit reachable from `HEAD` if `base` is `None`
    /// - Sets [`Commit::reverted_by`] from [`Reverts`]
    pub fn get(&self, base: Option<&Version>) -> Result<Vec<Commit>, Report<CommitProviderError>> {
        let git_commits = self
            .commits
            .get_commits(base.cloned())
            .change_context(CommitProviderError::GetCommits)?;
        let reverts = Reverts::from(git_commits.as_slice());
        let commits = git_commits
            .iter()
            .map(|git_commit| Commit {
                reverted_by: reverts.get(&git_commit.hash),
                ..self.parser.parse(git_commit)
            })
            .collect();
        Ok(commits)
    }
}

/// Errors returned by [`CommitProvider`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CommitProviderError {
    /// Unable to get the commits from git.
    #[error("Unable to get the commits since the last release")]
    GetCommits,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`CommitProvider::get`] marks only reverted commits, with the hash of their revert.
    #[test]
    fn commit_provider_get() {
        // Arrange
        let provider = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<CommitProvider>();
        // Act
        let commits = provider.get(None).expect("should get commits");
        // Assert
        let reverted: BTreeMap<CommitHash, CommitHash> = commits
            .iter()
            .filter_map(|commit| Some((commit.hash, commit.reverted_by?)))
            .collect();
        assert_eq!(
            reverted,
            BTreeMap::from([
                (
                    CommitHash::fixture("ddddddd4"),
                    CommitHash::fixture("eeeeeee5")
                ),
                (
                    CommitHash::fixture("6666666f"),
                    CommitHash::fixture("7777777a")
                ),
            ])
        );
    }
}
