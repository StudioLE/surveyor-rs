//! [`WriteManifest`] spy.

use crate::prelude::*;

/// [`WriteManifest`] spy recording each write.
#[derive(Default)]
pub struct WriteManifestSpy {
    /// Text written to each manifest by path.
    pub manifests: Mutex<BTreeMap<PathBuf, String>>,
}

impl WriteManifestSpy {
    /// Get the text written to each manifest by path.
    pub fn get_manifests(&self) -> BTreeMap<PathBuf, String> {
        self.manifests
            .lock()
            .expect("lock should not be poisoned")
            .clone()
    }
}

impl WriteManifest for WriteManifestSpy {
    fn write_manifest(&self, path: &Path, text: &str) -> Result<(), Report<WriteManifestError>> {
        self.manifests
            .lock()
            .expect("lock should not be poisoned")
            .insert(path.to_path_buf(), text.to_owned());
        Ok(())
    }
}

impl FromServices for WriteManifestSpy {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
