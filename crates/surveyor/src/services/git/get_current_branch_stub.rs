//! [`GetCurrentBranch`] stub.

use crate::prelude::*;

/// [`GetCurrentBranch`] stub.
pub struct GetCurrentBranchStub {
    /// Name of the checked out branch.
    ///
    /// Default: `main`
    pub branch: String,
}

impl Default for GetCurrentBranchStub {
    fn default() -> Self {
        Self {
            branch: "main".to_owned(),
        }
    }
}

impl GetCurrentBranch for GetCurrentBranchStub {
    fn get_current_branch(&self) -> Result<String, Report<GetCurrentBranchError>> {
        Ok(self.branch.clone())
    }
}

impl FromServices for GetCurrentBranchStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
