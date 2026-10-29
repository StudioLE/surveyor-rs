//! [`GetRepoName`] stub.

use crate::prelude::*;

/// [`GetRepoName`] stub.
pub struct GetRepoNameStub {
    /// GitHub repo, or the error to fail with.
    ///
    /// Default: [`RepoName::fixture`]
    pub repo_name: Result<RepoName, GetRepoNameError>,
}

impl Default for GetRepoNameStub {
    fn default() -> Self {
        Self {
            repo_name: Ok(RepoName::fixture()),
        }
    }
}

impl GetRepoName for GetRepoNameStub {
    fn get_repo_name(&self) -> Result<RepoName, Report<GetRepoNameError>> {
        self.repo_name.clone().map_err(Report::new)
    }
}

impl FromServices for GetRepoNameStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
