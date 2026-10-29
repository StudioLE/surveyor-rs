//! Version tags in the repo.

use crate::prelude::*;

/// Versions parsed from `v` prefixed semver tags, each list in ascending order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VersionTags {
    /// Every version tag in the repo.
    pub all: Vec<Version>,
    /// Version tags reachable from `HEAD`.
    pub merged: Vec<Version>,
    /// Version tags pointing at `HEAD`.
    pub at_head: Vec<Version>,
}

#[cfg(test)]
impl VersionTags {
    /// Create [`VersionTags`] with `v1.0.0` merged and nothing at `HEAD`.
    pub(crate) fn fixture() -> Self {
        Self {
            all: vec![Version::new(1, 0, 0)],
            merged: vec![Version::new(1, 0, 0)],
            at_head: Vec::new(),
        }
    }
}
