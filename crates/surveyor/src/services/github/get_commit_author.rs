//! Lookup of the GitHub login that authored a commit.

use crate::prelude::*;

/// Get the GitHub login that authored a commit.
pub trait GetCommitAuthor: Send + Sync {
    /// Get the login of the GitHub user matching the author of `hash` in `repo`.
    ///
    /// - Fails if GitHub matches no user to the author email
    fn get_commit_author(
        &self,
        repo: &RepoName,
        hash: &CommitHash,
    ) -> Result<String, Report<GetCommitAuthorError>>;
}

impl GetCommitAuthor for GhAdapter {
    fn get_commit_author(
        &self,
        repo: &RepoName,
        hash: &CommitHash,
    ) -> Result<String, Report<GetCommitAuthorError>> {
        let path = format!("repos/{repo}/commits/{hash}");
        self.run(&["api", &path, "--jq", ".author.login"])
            .change_context(GetCommitAuthorError::Gh)
    }
}

/// Errors returned by [`GetCommitAuthor::get_commit_author`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetCommitAuthorError {
    /// Unable to get the commit author from `gh`.
    #[error("Unable to get the GitHub login of the commit author")]
    Gh,
}
