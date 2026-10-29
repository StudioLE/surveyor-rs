//! [`GetLastCommitTouching`] spy.

use crate::prelude::*;

/// [`GetLastCommitTouching`] spy recording each lookup.
#[derive(Default)]
pub struct GetLastCommitTouchingSpy {
    /// Last commit returned for any paths.
    ///
    /// Default: `None`
    pub hash: Option<CommitHash>,
    /// Excludes of each lookup by its paths.
    pub lookups: Mutex<BTreeMap<Vec<PathBuf>, Vec<PathBuf>>>,
}

impl GetLastCommitTouchingSpy {
    /// Get the excludes of each lookup by its paths.
    pub fn get_lookups(&self) -> BTreeMap<Vec<PathBuf>, Vec<PathBuf>> {
        self.lookups
            .lock()
            .expect("lock should not be poisoned")
            .clone()
    }
}

impl GetLastCommitTouching for GetLastCommitTouchingSpy {
    fn get_last_commit_touching(
        &self,
        paths: &[PathBuf],
        excludes: &[PathBuf],
    ) -> Result<Option<CommitHash>, Report<GetLastCommitTouchingError>> {
        self.lookups
            .lock()
            .expect("lock should not be poisoned")
            .insert(paths.to_vec(), excludes.to_vec());
        Ok(self.hash)
    }
}

impl FromServices for GetLastCommitTouchingSpy {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
