//! [`GetTagsContaining`] spy.

use crate::prelude::*;

/// [`GetTagsContaining`] spy recording each lookup.
#[derive(Default)]
pub struct GetTagsContainingSpy {
    /// Versions returned for any commit.
    ///
    /// Default: empty
    pub tags: Vec<Version>,
    /// Commits looked up, in order.
    pub hashes: Mutex<Vec<CommitHash>>,
}

impl GetTagsContainingSpy {
    /// Get the commits looked up, in order.
    pub fn get_hashes(&self) -> Vec<CommitHash> {
        self.hashes
            .lock()
            .expect("lock should not be poisoned")
            .clone()
    }
}

impl GetTagsContaining for GetTagsContainingSpy {
    fn get_tags_containing(
        &self,
        hash: &CommitHash,
    ) -> Result<Vec<Version>, Report<GetTagsContainingError>> {
        self.hashes
            .lock()
            .expect("lock should not be poisoned")
            .push(*hash);
        Ok(self.tags.clone())
    }
}

impl FromServices for GetTagsContainingSpy {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
