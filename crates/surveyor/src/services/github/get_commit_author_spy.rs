//! [`GetCommitAuthor`] spy.

use crate::prelude::*;

/// [`GetCommitAuthor`] spy recording each lookup.
pub struct GetCommitAuthorSpy {
    /// GitHub login of the author of each commit.
    ///
    /// - Fails with [`GetCommitAuthorError::Gh`] for any other commit
    ///
    /// Default: `ccccccc3` from [`GitCommit::fixture_history`] is `ada`
    pub logins: HashMap<CommitHash, String>,
    /// Commits looked up, in order.
    pub hashes: Mutex<Vec<CommitHash>>,
}

impl GetCommitAuthorSpy {
    /// Get the commits looked up, in order.
    pub fn get_hashes(&self) -> Vec<CommitHash> {
        self.hashes
            .lock()
            .expect("lock should not be poisoned")
            .clone()
    }
}

impl Default for GetCommitAuthorSpy {
    fn default() -> Self {
        Self {
            logins: HashMap::from([(CommitHash::fixture("ccccccc3"), "ada".to_owned())]),
            hashes: Mutex::default(),
        }
    }
}

impl GetCommitAuthor for GetCommitAuthorSpy {
    fn get_commit_author(
        &self,
        _: &RepoName,
        hash: &CommitHash,
    ) -> Result<String, Report<GetCommitAuthorError>> {
        self.hashes
            .lock()
            .expect("lock should not be poisoned")
            .push(*hash);
        self.logins
            .get(hash)
            .cloned()
            .ok_or_else(|| Report::new(GetCommitAuthorError::Gh))
    }
}

impl FromServices for GetCommitAuthorSpy {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
