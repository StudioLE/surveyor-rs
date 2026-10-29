//! Dispatch of the selected subcommand.

use crate::prelude::*;

/// Dispatch the selected [`Subcommand`] to its handler.
///
/// - Matches by hand because [`VersionHandler::run`] and [`NotesHandler::run`] return
///   different error types
#[derive(FromServices)]
pub struct SubcommandHandler {
    cli: Arc<CliArgs>,
    version: Arc<VersionHandler>,
    notes: Arc<NotesHandler>,
}

impl SubcommandHandler {
    /// Execute the selected subcommand.
    pub fn run(&self) -> Result<(), StructuredError> {
        let command = self.cli.command.clone();
        match command {
            Subcommand::Version(args) => Ok(self.version.run(args)?),
            Subcommand::Notes => Ok(self.notes.run()?),
        }
    }
}
