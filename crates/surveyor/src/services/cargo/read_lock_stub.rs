//! [`ReadLock`] stub.

use crate::prelude::*;

/// [`ReadLock`] stub.
pub struct ReadLockStub {
    /// Text of the lock file at any path, or `None` if it doesn't exist.
    ///
    /// Default: `alpha` and `beta` at `0.0.0`
    pub lock: Option<String>,
}

impl Default for ReadLockStub {
    fn default() -> Self {
        let lock = "version = 4\n\n[[package]]\nname = \"alpha\"\nversion = \"0.0.0\"\n\n[[package]]\nname = \"beta\"\nversion = \"0.0.0\"\n";
        Self {
            lock: Some(lock.to_owned()),
        }
    }
}

impl ReadLock for ReadLockStub {
    fn read_lock(&self, _: &Path) -> Result<Option<String>, Report<ReadLockError>> {
        Ok(self.lock.clone())
    }
}

impl FromServices for ReadLockStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
