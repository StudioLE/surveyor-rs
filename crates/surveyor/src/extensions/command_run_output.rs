//! Running a [`Command`] to completion with error reports.

use crate::prelude::*;
use std::process::{Command, Output};

/// Run a [`Command`] to completion, returning an error report on failure.
pub trait RunOutput {
    /// Run the command and return its output if it exits successfully.
    ///
    /// - Logs the command line at trace before running
    /// - Fails with `spawn` if the command cannot start
    /// - Fails with `exit` if the command exits with an error, attaching stderr, stdout, and exit code
    /// - Attaches the command line on failure
    fn run_output<C: Error + Send + Sync + 'static>(
        &mut self,
        spawn: C,
        exit: C,
    ) -> Result<Output, Report<C>>;
}

impl RunOutput for Command {
    fn run_output<C: Error + Send + Sync + 'static>(
        &mut self,
        spawn: C,
        exit: C,
    ) -> Result<Output, Report<C>> {
        let command = self.to_command_line();
        trace!(%command, "Running");
        self.output()
            .change_context(spawn)
            .attach("command", &command)?
            .ok_or_report(exit)
            .attach("command", &command)
    }
}
