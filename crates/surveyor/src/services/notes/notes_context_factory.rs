//! Creation of the data passed to the release notes template.

use crate::prelude::*;

/// Create the data passed to the release notes template.
#[derive(FromServices)]
pub struct NotesContextFactory {
    authors: Arc<AuthorResolver>,
}

impl NotesContextFactory {
    /// Create a new [`NotesContext`] from `commits`, grouping them into sections.
    ///
    /// - Reverses newest first `commits` so the notes read oldest first
    /// - Sets GitHub logins with [`AuthorResolver::resolve`]
    pub fn create(&self, commits: &[Commit]) -> NotesContext {
        let mut commits = commits.to_vec();
        commits.reverse();
        for commit in &mut commits {
            commit.author.github = self.authors.resolve(commit);
        }
        let sections = SectionsContext::from_commits(&commits);
        NotesContext { commits, sections }
    }
}
