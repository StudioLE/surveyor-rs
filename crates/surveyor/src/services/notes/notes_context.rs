//! Data passed to the release notes template.

use crate::prelude::*;

/// Data passed to the release notes template.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NotesContext {
    /// Every commit since the last full release, oldest first.
    pub commits: Vec<Commit>,
    /// Commits grouped into sections.
    pub sections: SectionsContext,
}
