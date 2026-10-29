//! Reading of Cargo manifests.

use crate::prelude::*;
use std::fs;

/// Read a Cargo manifest.
pub trait ReadManifest: Send + Sync {
    /// Read the text of the manifest at `path`.
    fn read_manifest(&self, path: &Path) -> Result<String, Report<ReadManifestError>>;
}

impl ReadManifest for CargoAdapter {
    fn read_manifest(&self, path: &Path) -> Result<String, Report<ReadManifestError>> {
        fs::read_to_string(path)
            .change_context(ReadManifestError::Read)
            .attach_path(path)
    }
}

/// Errors returned by [`ReadManifest::read_manifest`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ReadManifestError {
    /// Unable to read the manifest.
    #[error("Unable to read manifest")]
    Read,
}
