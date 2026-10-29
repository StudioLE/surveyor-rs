//! Lookup of the GitHub owner and repo.

use crate::prelude::*;

/// Get the GitHub owner and repo.
pub trait GetRepoName: Send + Sync {
    /// Get the owner and repo, such as `StudioLE/surveyor-rs`.
    ///
    /// - Reads `GITHUB_REPOSITORY`, else runs `gh repo view`
    fn get_repo_name(&self) -> Result<RepoName, Report<GetRepoNameError>>;
}

impl GetRepoName for GhAdapter {
    fn get_repo_name(&self) -> Result<RepoName, Report<GetRepoNameError>> {
        if let Some(repository) = &self.environment.repository {
            return Ok(repository.clone());
        }
        let output = self
            .run(&[
                "repo",
                "view",
                "--json",
                "nameWithOwner",
                "--jq",
                ".nameWithOwner",
            ])
            .change_context(GetRepoNameError::Gh)?;
        RepoName::from_str(&output).change_context(GetRepoNameError::Parse)
    }
}

/// Errors returned by [`GetRepoName::get_repo_name`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetRepoNameError {
    /// Unable to get the repo from `gh`.
    #[error("Unable to get the GitHub repo name")]
    Gh,
    /// `gh` printed a repo that is not `<owner>/<name>`.
    #[error("Unable to parse the GitHub repo name")]
    Parse,
}
