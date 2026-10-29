//! [`ReadManifest`] stub.

use crate::prelude::*;

/// [`ReadManifest`] stub.
pub struct ReadManifestStub {
    /// Text of each manifest by path.
    ///
    /// - Fails with [`ReadManifestError::Read`] for any other path
    ///
    /// Default: every [`Workspace::fixture`] manifest is `[package]` with `version = "0.0.0"`
    pub manifests: BTreeMap<PathBuf, String>,
}

impl Default for ReadManifestStub {
    fn default() -> Self {
        let text = "[package]\nversion = \"0.0.0\"\n";
        let paths = [
            "/repo/Cargo.toml",
            "/repo/crates/alpha/Cargo.toml",
            "/repo/crates/beta/Cargo.toml",
        ];
        Self {
            manifests: paths
                .into_iter()
                .map(|path| (PathBuf::from(path), text.to_owned()))
                .collect(),
        }
    }
}

impl ReadManifest for ReadManifestStub {
    fn read_manifest(&self, path: &Path) -> Result<String, Report<ReadManifestError>> {
        self.manifests
            .get(path)
            .cloned()
            .ok_or_else(|| Report::new(ReadManifestError::Read).attach_path(path))
    }
}

impl FromServices for ReadManifestStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
