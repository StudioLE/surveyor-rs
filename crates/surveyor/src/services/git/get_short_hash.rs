//! Lookup of the short hash of `HEAD`.

use crate::prelude::*;

/// Get the short hash of `HEAD`.
pub trait GetShortHash: Send + Sync {
    /// Get the short hash of `HEAD` with at least [`ShortCommitHash::MIN_LENGTH`] characters.
    ///
    /// - Git lengthens it if a shorter hash would be ambiguous
    fn get_short_hash(&self) -> Result<ShortCommitHash, Report<GetShortHashError>>;
}

impl GetShortHash for GitAdapter {
    fn get_short_hash(&self) -> Result<ShortCommitHash, Report<GetShortHashError>> {
        let short = format!("--short={}", ShortCommitHash::MIN_LENGTH);
        let output = self
            .run(["rev-parse", &short, "HEAD"])
            .change_context(GetShortHashError::Git)?;
        ShortCommitHash::from_str(output.trim()).change_context(GetShortHashError::Hash)
    }
}

/// Errors returned by [`GetShortHash::get_short_hash`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetShortHashError {
    /// Unable to run `git rev-parse`.
    #[error("Unable to get the short hash of HEAD")]
    Git,
    /// `git rev-parse` printed an invalid short hash.
    #[error("Unable to parse the short hash of HEAD")]
    Hash,
}
