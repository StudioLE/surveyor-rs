//! Comparison of version cores.

use crate::prelude::*;

/// Compare the core of two [`Version`] values.
pub trait HasSameCore {
    /// Do both versions have the same major, minor, and patch?
    ///
    /// - Ignores pre-release identifiers and build metadata
    fn has_same_core(&self, other: &Version) -> bool;
}

impl HasSameCore for Version {
    fn has_same_core(&self, other: &Version) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`HasSameCore::has_same_core`] ignores the pre-release and compares each position.
    #[test]
    fn has_same_core_has_same_core() {
        let version = Version::fixture("1.2.3-alpha.1");
        assert!(version.has_same_core(&Version::fixture("1.2.3")));
        assert!(!version.has_same_core(&Version::fixture("1.2.4")));
        assert!(!version.has_same_core(&Version::fixture("1.3.3")));
        assert!(!version.has_same_core(&Version::fixture("2.2.3")));
    }
}
