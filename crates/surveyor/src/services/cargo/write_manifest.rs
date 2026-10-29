//! Writing of Cargo manifests.

use crate::prelude::*;
use std::fs;

/// Write a Cargo manifest.
pub trait WriteManifest: Send + Sync {
    /// Replace the manifest at `path` with `text`.
    fn write_manifest(&self, path: &Path, text: &str) -> Result<(), Report<WriteManifestError>>;
}

impl WriteManifest for CargoAdapter {
    fn write_manifest(&self, path: &Path, text: &str) -> Result<(), Report<WriteManifestError>> {
        debug!(path = %path.display(), "Writing manifest");
        fs::write(path, text)
            .change_context(WriteManifestError::Write)
            .attach_path(path)
    }
}

/// Errors returned by [`WriteManifest::write_manifest`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WriteManifestError {
    /// Unable to write the manifest.
    #[error("Unable to write manifest")]
    Write,
}
