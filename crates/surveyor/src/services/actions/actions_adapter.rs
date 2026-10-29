//! Adapter for GitHub Actions step files.

use crate::prelude::*;
use std::fs::OpenOptions;

/// Adapter appending to the GitHub Actions step files named in the [`ActionsEnvironment`].
#[derive(FromServices)]
pub struct ActionsAdapter {
    /// Paths of the step files.
    pub(super) environment: Arc<ActionsEnvironment>,
}

impl ActionsAdapter {
    /// Append `text` to the file at `path`, creating it if missing.
    pub(super) fn append(path: &Path, text: &str) -> Result<(), Report<ActionsAdapterError>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .change_context(ActionsAdapterError::Open)
            .attach_path(path)?;
        file.write_all(text.as_bytes())
            .change_context(ActionsAdapterError::Write)
            .attach_path(path)
    }
}

/// Errors returned by [`ActionsAdapter`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ActionsAdapterError {
    /// Unable to open the file.
    #[error("Unable to open GitHub Actions file")]
    Open,
    /// Unable to write the file.
    #[error("Unable to write GitHub Actions file")]
    Write,
}
