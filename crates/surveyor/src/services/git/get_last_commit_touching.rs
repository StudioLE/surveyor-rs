//! Lookup of the last commit touching a set of paths.

use crate::prelude::*;
use std::ffi::OsString;

/// Get the last commit touching a set of paths.
pub trait GetLastCommitTouching: Send + Sync {
    /// Get the hash of the last commit reachable from `HEAD` touching `paths` but not `excludes`.
    ///
    /// - Returns `None` if no commit touches them
    fn get_last_commit_touching(
        &self,
        paths: &[PathBuf],
        excludes: &[PathBuf],
    ) -> Result<Option<CommitHash>, Report<GetLastCommitTouchingError>>;
}

impl GetLastCommitTouching for GitAdapter {
    fn get_last_commit_touching(
        &self,
        paths: &[PathBuf],
        excludes: &[PathBuf],
    ) -> Result<Option<CommitHash>, Report<GetLastCommitTouchingError>> {
        let mut args: Vec<OsString> = ["log", "-1", "--format=%H", "HEAD", "--"]
            .into_iter()
            .map(OsString::from)
            .collect();
        args.extend(paths.iter().map(OsString::from));
        args.extend(excludes.iter().map(|path| {
            let mut exclude = OsString::from(":(exclude)");
            exclude.push(path);
            exclude
        }));
        let output = self
            .run(args)
            .change_context(GetLastCommitTouchingError::Git)?;
        let hash = output.trim();
        if hash.is_empty() {
            return Ok(None);
        }
        let hash = CommitHash::from_str(hash).change_context(GetLastCommitTouchingError::Hash)?;
        Ok(Some(hash))
    }
}

/// Errors returned by [`GetLastCommitTouching::get_last_commit_touching`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetLastCommitTouchingError {
    /// Unable to run `git log`.
    #[error("Unable to get the last commit touching the crate")]
    Git,
    /// `git log` printed an invalid commit hash.
    #[error("Unable to parse the hash of the last commit touching the crate")]
    Hash,
}
