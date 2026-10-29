//! Test versions parsed from literals.

use crate::prelude::*;

/// Create a [`Version`] for tests.
pub trait VersionFixture {
    /// Create a [`Version`] by parsing `text`, such as `1.1.0-alpha.1`.
    fn fixture(text: &str) -> Self;
}

impl VersionFixture for Version {
    fn fixture(text: &str) -> Self {
        Self::from_str(text).expect("fixture version should parse")
    }
}
