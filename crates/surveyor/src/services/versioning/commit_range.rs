//! Commits since the last full release.

use crate::prelude::*;

/// Commits since the last full release, with the tags they were measured against.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitRange {
    /// Version tags in the repo.
    pub tags: VersionTags,
    /// Newest full release reachable from `HEAD`.
    ///
    /// - `None` if there is no full release, so the range is every commit
    pub base: Option<Version>,
    /// Non-merge commits in `v<base>..HEAD`, newest first.
    pub commits: Vec<Commit>,
}
