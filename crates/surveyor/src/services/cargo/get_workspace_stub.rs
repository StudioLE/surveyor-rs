//! [`GetWorkspace`] stub.

use crate::prelude::*;

/// [`GetWorkspace`] stub.
pub struct GetWorkspaceStub {
    /// Cargo workspace of the repo.
    ///
    /// Default: [`Workspace::fixture`]
    pub workspace: Workspace,
}

impl Default for GetWorkspaceStub {
    fn default() -> Self {
        Self {
            workspace: Workspace::fixture(),
        }
    }
}

impl GetWorkspace for GetWorkspaceStub {
    fn get_workspace(&self) -> Result<Workspace, Report<GetWorkspaceError>> {
        Ok(self.workspace.clone())
    }
}

impl FromServices for GetWorkspaceStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
