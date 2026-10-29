//! Adapter wrapping the `gh` CLI.

use crate::prelude::*;
use std::process::Command;

/// Adapter wrapping the `gh` CLI, run in the [`WorkingDirectory`].
#[derive(FromServices)]
pub struct GhAdapter {
    /// Source of `GITHUB_REPOSITORY`.
    pub(super) environment: Arc<ActionsEnvironment>,
    working_directory: Arc<WorkingDirectory>,
}

impl GhAdapter {
    /// Run `gh` with `args` and return its trimmed stdout.
    ///
    /// - Fails if the output is not UTF-8, or is empty or `null`
    /// - Attaches the command line, stderr, and exit code on failure
    pub(super) fn run(&self, args: &[&str]) -> Result<String, Report<GhAdapterError>> {
        let mut command = Command::new("gh");
        command
            .args(args)
            .current_dir(self.working_directory.path());
        let output = command.run_output(GhAdapterError::Spawn, GhAdapterError::Exit)?;
        let text = String::from_utf8(output.stdout)
            .change_context(GhAdapterError::Utf8)
            .attach("command", command.to_command_line())?;
        let text = text.trim();
        if text.is_empty() || text == "null" {
            return Err(
                Report::new(GhAdapterError::Empty).attach("command", command.to_command_line())
            );
        }
        Ok(text.to_owned())
    }
}

/// Errors returned by [`GhAdapter`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GhAdapterError {
    /// Unable to start `gh`.
    #[error("Unable to start gh")]
    Spawn,
    /// `gh` exited with an error.
    #[error("GitHub CLI exited with an error")]
    Exit,
    /// `gh` output is not UTF-8.
    #[error("GitHub CLI output is not UTF-8")]
    Utf8,
    /// `gh` printed nothing.
    #[error("GitHub CLI returned no value")]
    Empty,
}
