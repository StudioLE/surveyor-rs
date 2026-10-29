//! Reading of `Cargo.lock`.

use crate::prelude::*;
use std::fs;

/// Read a `Cargo.lock`.
pub trait ReadLock: Send + Sync {
    /// Read the text of the lock file at `path`, or `None` if it doesn't exist.
    fn read_lock(&self, path: &Path) -> Result<Option<String>, Report<ReadLockError>>;
}

impl ReadLock for CargoAdapter {
    fn read_lock(&self, path: &Path) -> Result<Option<String>, Report<ReadLockError>> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(Report::new(error)
                .change_context(ReadLockError::Read)
                .attach_path(path)),
        }
    }
}

/// Errors returned by [`ReadLock::read_lock`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ReadLockError {
    /// Unable to read the lock file.
    #[error("Unable to read lock file")]
    Read,
}
