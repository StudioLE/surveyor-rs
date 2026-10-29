//! Writing of `Cargo.lock`.

use crate::prelude::*;
use std::fs;

/// Write a `Cargo.lock`.
pub trait WriteLock: Send + Sync {
    /// Replace the lock file at `path` with `text`.
    fn write_lock(&self, path: &Path, text: &str) -> Result<(), Report<WriteLockError>>;
}

impl WriteLock for CargoAdapter {
    fn write_lock(&self, path: &Path, text: &str) -> Result<(), Report<WriteLockError>> {
        debug!(path = %path.display(), "Writing lock file");
        fs::write(path, text)
            .change_context(WriteLockError::Write)
            .attach_path(path)
    }
}

/// Errors returned by [`WriteLock::write_lock`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WriteLockError {
    /// Unable to write the lock file.
    #[error("Unable to write lock file")]
    Write,
}
