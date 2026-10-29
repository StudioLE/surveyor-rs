//! Author of a commit.

use crate::prelude::*;

/// Author of a commit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Author {
    /// Author name from git.
    pub name: String,
    /// Author email from git.
    pub email: String,
    /// GitHub login if matched to a user.
    ///
    /// - `None` if [`AuthorResolver::resolve`] matched no user or hasn't run
    pub github: Option<String>,
}
