//! Lookup of the version tags in the repo.

use crate::prelude::*;

/// Get the version tags in the repo.
pub trait GetVersionTags: Send + Sync {
    /// Get every version tag, those reachable from `HEAD`, and those pointing at `HEAD`.
    fn get_version_tags(&self) -> Result<VersionTags, Report<GetVersionTagsError>>;
}

impl GetVersionTags for GitAdapter {
    fn get_version_tags(&self) -> Result<VersionTags, Report<GetVersionTagsError>> {
        let all = self
            .get_tags(&[])
            .change_context(GetVersionTagsError::All)?;
        let merged = self
            .get_tags(&["--merged", "HEAD"])
            .change_context(GetVersionTagsError::Merged)?;
        let at_head = self
            .get_tags(&["--points-at", "HEAD"])
            .change_context(GetVersionTagsError::AtHead)?;
        Ok(VersionTags {
            all,
            merged,
            at_head,
        })
    }
}

/// Errors returned by [`GetVersionTags::get_version_tags`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetVersionTagsError {
    /// Unable to list every tag.
    #[error("Unable to list version tags")]
    All,
    /// Unable to list the tags reachable from `HEAD`.
    #[error("Unable to list version tags reachable from HEAD")]
    Merged,
    /// Unable to list the tags pointing at `HEAD`.
    #[error("Unable to list version tags pointing at HEAD")]
    AtHead,
}
