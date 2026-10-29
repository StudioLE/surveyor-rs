//! Handler for the notes subcommand.

use crate::prelude::*;

/// Execute the notes subcommand.
#[derive(FromServices)]
pub struct NotesHandler {
    ranges: Arc<CommitRangeProvider>,
    notes: Arc<NotesRenderer>,
    console: Arc<dyn WriteConsole>,
    summary: Arc<dyn WriteSummary>,
}

impl NotesHandler {
    /// Render release notes for the commits since the last full release to stdout.
    ///
    /// - Prints nothing if there are no commits
    /// - Appends the notes to the step summary if `GITHUB_STEP_SUMMARY` is set
    pub fn run(&self) -> Result<(), Report<NotesHandlerError>> {
        let range = self.ranges.get().change_context(NotesHandlerError::Range)?;
        if range.commits.is_empty() {
            info!("No commits since the last release, so there are no notes");
            return Ok(());
        }
        let notes = self
            .notes
            .render(&range.commits)
            .change_context(NotesHandlerError::Render)?;
        self.console
            .write(&notes)
            .change_context(NotesHandlerError::Console)?;
        self.summary
            .write_summary(&notes)
            .change_context(NotesHandlerError::Summary)
    }
}

/// Errors returned by [`NotesHandler`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum NotesHandlerError {
    /// Unable to get the commits since the last release.
    #[error("Unable to get the commits since the last release")]
    Range,
    /// Unable to render the notes.
    #[error("Unable to render the release notes")]
    Render,
    /// Unable to write the notes to stdout.
    #[error("Unable to write the release notes to stdout")]
    Console,
    /// Unable to write the step summary.
    #[error("Unable to write the release notes to the step summary")]
    Summary,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`NotesHandler::run`] writes the notes to stdout and appends them to the step summary.
    #[test]
    fn notes_handler_run() {
        // Arrange
        let services = ServiceBuilder::new().with_test_services().build();
        let handler = services.expect::<NotesHandler>();
        // Act
        handler.run().expect("should render notes");
        // Assert
        let summaries = services.expect::<WriteSummarySpy>().get_summaries();
        assert_eq!(summaries.len(), 1);
        let notes = summaries.join("");
        assert!(notes.contains("add flag"), "{notes}");
        let stdout = services.expect::<WriteConsoleSpy>().get_output();
        assert_eq!(stdout, notes);
    }

    /// [`NotesHandler::run`] writes nothing when there are no commits.
    #[test]
    fn notes_handler_run_no_commits() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: Vec::new(),
            })
            .build();
        let handler = services.expect::<NotesHandler>();
        // Act
        handler.run().expect("no commits should succeed");
        // Assert
        let summaries = services.expect::<WriteSummarySpy>().get_summaries();
        let stdout = services.expect::<WriteConsoleSpy>().get_output();
        assert_eq!(summaries, Vec::<String>::new());
        assert_eq!(stdout, "");
    }
}
