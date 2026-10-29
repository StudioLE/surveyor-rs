//! Version chosen for a workspace member.

use crate::prelude::*;

/// Version chosen for a workspace member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrateVersion {
    /// Package name.
    pub name: String,
    /// Version to write into the crate manifest.
    pub version: Version,
    /// Does the crate get the repo version, rather than keep a released version?
    pub bumped: bool,
    /// Can the crate be published?
    pub publish: bool,
}

#[cfg(any(test, feature = "testing"))]
impl CrateVersion {
    /// Create a bumped, publishable [`CrateVersion`] of `alpha` at `2.0.0`.
    #[must_use]
    pub fn fixture() -> Self {
        Self {
            name: "alpha".to_owned(),
            version: Version::new(2, 0, 0),
            bumped: true,
            publish: true,
        }
    }
}
