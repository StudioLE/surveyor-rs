//! Lookup of the checked out branch.

use crate::prelude::*;

/// Get the checked out branch.
pub trait GetCurrentBranch: Send + Sync {
    /// Get the name of the checked out branch.
    ///
    /// - Fails if `HEAD` is detached
    fn get_current_branch(&self) -> Result<String, Report<GetCurrentBranchError>>;
}

impl GetCurrentBranch for GitAdapter {
    fn get_current_branch(&self) -> Result<String, Report<GetCurrentBranchError>> {
        let output = self
            .run(["branch", "--show-current"])
            .change_context(GetCurrentBranchError::Git)?;
        let branch = output.trim();
        if branch.is_empty() {
            return Err(Report::new(GetCurrentBranchError::Detached));
        }
        Ok(branch.to_owned())
    }
}

/// Errors returned by [`GetCurrentBranch::get_current_branch`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetCurrentBranchError {
    /// Unable to run `git branch`.
    #[error("Unable to get the current branch")]
    Git,
    /// `HEAD` is not on a branch.
    #[error("HEAD is detached, so there is no branch to pick a channel from")]
    Detached,
}
