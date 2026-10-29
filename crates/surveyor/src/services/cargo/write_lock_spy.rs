//! [`WriteLock`] spy.

use crate::prelude::*;

/// [`WriteLock`] spy recording each write.
#[derive(Default)]
pub struct WriteLockSpy {
    /// Text written to each lock file by path.
    pub locks: Mutex<BTreeMap<PathBuf, String>>,
}

impl WriteLockSpy {
    /// Get the text written to each lock file by path.
    pub fn get_locks(&self) -> BTreeMap<PathBuf, String> {
        self.locks
            .lock()
            .expect("lock should not be poisoned")
            .clone()
    }
}

impl WriteLock for WriteLockSpy {
    fn write_lock(&self, path: &Path, text: &str) -> Result<(), Report<WriteLockError>> {
        self.locks
            .lock()
            .expect("lock should not be poisoned")
            .insert(path.to_path_buf(), text.to_owned());
        Ok(())
    }
}

impl FromServices for WriteLockSpy {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
