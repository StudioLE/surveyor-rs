//! Type of a conventional commit.

use crate::prelude::*;

/// Type of a conventional commit and the version change it implies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitType {
    /// Configured type name if matched, else the raw type.
    pub kind: String,
    /// Version change implied by the type.
    pub change: Change,
}
