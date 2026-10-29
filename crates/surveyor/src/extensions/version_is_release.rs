//! Check for full release versions.

use crate::prelude::*;

/// Check whether a [`Version`] is a full release.
pub trait IsRelease {
    /// Is the version a full release, with no pre-release identifier?
    ///
    /// - Ignores build metadata
    fn is_release(&self) -> bool;
}

impl IsRelease for Version {
    fn is_release(&self) -> bool {
        self.pre.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`IsRelease::is_release`] accepts build metadata and rejects a pre-release.
    #[test]
    fn is_release_is_release() {
        assert!(Version::fixture("1.2.0").is_release());
        assert!(Version::fixture("1.2.0+dirty").is_release());
        assert!(!Version::fixture("1.2.0-alpha.1").is_release());
    }
}
