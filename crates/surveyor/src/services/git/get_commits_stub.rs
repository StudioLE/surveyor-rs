//! [`GetCommits`] stub.

use crate::prelude::*;

/// [`GetCommits`] stub.
pub struct GetCommitsStub {
    /// Commits returned for any base.
    ///
    /// Default: [`GitCommit::fixture_history`]
    pub commits: Vec<GitCommit>,
}

impl Default for GetCommitsStub {
    fn default() -> Self {
        Self {
            commits: GitCommit::fixture_history(),
        }
    }
}

impl GetCommits for GetCommitsStub {
    fn get_commits(&self, _: Option<Version>) -> Result<Vec<GitCommit>, Report<GetCommitsError>> {
        Ok(self.commits.clone())
    }
}

impl FromServices for GetCommitsStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
