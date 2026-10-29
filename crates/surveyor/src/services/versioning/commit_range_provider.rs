//! Retrieval of the commits since the last full release.

use crate::prelude::*;

/// Get the commits since the last full release.
#[derive(FromServices)]
pub struct CommitRangeProvider {
    tags: Arc<dyn GetVersionTags>,
    commits: Arc<CommitProvider>,
}

impl CommitRangeProvider {
    /// Get the commits since the newest full release reachable from `HEAD`.
    pub fn get(&self) -> Result<CommitRange, Report<CommitRangeProviderError>> {
        let tags = self
            .tags
            .get_version_tags()
            .change_context(CommitRangeProviderError::Tags)?;
        let base = tags
            .merged
            .iter()
            .filter(|version| version.is_release())
            .max()
            .cloned();
        let commits = self
            .commits
            .get(base.as_ref())
            .change_context(CommitRangeProviderError::Commits)?;
        Ok(CommitRange {
            tags,
            base,
            commits,
        })
    }
}

/// Errors returned by [`CommitRangeProvider`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CommitRangeProviderError {
    /// Unable to get the version tags.
    #[error("Unable to get the version tags")]
    Tags,
    /// Unable to get the commits since the last release.
    #[error("Unable to get the commits since the last release")]
    Commits,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`CommitRangeProvider::get`] uses the newest merged full release as the base.
    #[test]
    fn commit_range_provider_get() {
        // Arrange
        let tags = VersionTags {
            all: vec![
                Version::fixture("1.0.0"),
                Version::fixture("1.1.0"),
                Version::fixture("1.2.0-alpha.1"),
                Version::fixture("2.0.0"),
            ],
            merged: vec![
                Version::fixture("1.0.0"),
                Version::fixture("1.1.0"),
                Version::fixture("1.2.0-alpha.1"),
            ],
            at_head: Vec::new(),
        };
        let git_commits = vec![GitCommit {
            message: "feat: one".to_owned(),
            ..GitCommit::fixture()
        }];
        let provider = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetVersionTagsStub { tags })
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<CommitRangeProvider>();
        // Act
        let range = provider.get().expect("should get range");
        // Assert
        assert_eq!(range.base, Some(Version::fixture("1.1.0")));
        assert_eq!(range.commits.len(), 1);
    }
}
