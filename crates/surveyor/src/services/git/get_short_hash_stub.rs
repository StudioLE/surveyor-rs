//! [`GetShortHash`] stub.

use crate::prelude::*;

/// [`GetShortHash`] stub.
pub struct GetShortHashStub {
    /// Short hash of `HEAD`.
    ///
    /// Default: `abcdef1`
    pub hash: ShortCommitHash,
}

impl Default for GetShortHashStub {
    fn default() -> Self {
        Self {
            hash: ShortCommitHash::from_str("abcdef1").expect("fixture short hash should parse"),
        }
    }
}

impl GetShortHash for GetShortHashStub {
    fn get_short_hash(&self) -> Result<ShortCommitHash, Report<GetShortHashError>> {
        Ok(self.hash.clone())
    }
}

impl FromServices for GetShortHashStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
