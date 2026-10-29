//! Lookup of the version tags containing a commit.

use crate::prelude::*;

/// Get the version tags containing a commit.
pub trait GetTagsContaining: Send + Sync {
    /// Get the versions of every version tag whose history contains `hash`, in ascending order.
    fn get_tags_containing(
        &self,
        hash: &CommitHash,
    ) -> Result<Vec<Version>, Report<GetTagsContainingError>>;
}

impl GetTagsContaining for GitAdapter {
    fn get_tags_containing(
        &self,
        hash: &CommitHash,
    ) -> Result<Vec<Version>, Report<GetTagsContainingError>> {
        self.get_tags(&["--contains", hash.as_str()])
            .change_context(GetTagsContainingError::Git)
    }
}

/// Errors returned by [`GetTagsContaining::get_tags_containing`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetTagsContainingError {
    /// Unable to run `git tag`.
    #[error("Unable to list the version tags containing a commit")]
    Git,
}
