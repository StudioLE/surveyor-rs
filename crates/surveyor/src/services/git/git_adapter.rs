//! Adapter wrapping the `git` CLI.

use crate::prelude::*;
use std::ffi::OsStr;
use std::process::Command;

/// Adapter wrapping the `git` CLI, run in the [`WorkingDirectory`].
#[derive(FromServices)]
pub struct GitAdapter {
    working_directory: Arc<WorkingDirectory>,
}

impl GitAdapter {
    /// Run `git` with `args` and return its stdout.
    ///
    /// - Attaches the command line, stderr, and exit code on failure
    pub(super) fn run<T>(&self, args: T) -> Result<String, Report<GitAdapterError>>
    where
        T: IntoIterator,
        T::Item: AsRef<OsStr>,
    {
        let mut command = Command::new("git");
        command
            .args(args)
            .current_dir(self.working_directory.path());
        let output = command.run_output(GitAdapterError::Spawn, GitAdapterError::Exit)?;
        String::from_utf8(output.stdout)
            .change_context(GitAdapterError::Utf8)
            .attach("command", command.to_command_line())
    }

    /// Get the `v` prefixed semver tags matching `filter`, in ascending order.
    ///
    /// - Logs and skips tags that are not `v` followed by valid semver
    pub(super) fn get_tags(
        &self,
        filter: &[&str],
    ) -> Result<Vec<Version>, Report<GitAdapterError>> {
        let args = ["tag", "--list", "v*"].iter().chain(filter);
        let output = self.run(args)?;
        let mut versions: Vec<Version> = output
            .lines()
            .filter_map(|tag| {
                let parsed = tag
                    .strip_prefix('v')
                    .and_then(|version| Version::from_str(version).ok());
                if parsed.is_none() {
                    debug!(tag, "Ignoring tag that is not a version");
                }
                parsed
            })
            .collect();
        versions.sort();
        Ok(versions)
    }
}

/// Errors returned by [`GitAdapter`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GitAdapterError {
    /// Unable to start `git`.
    #[error("Unable to start git")]
    Spawn,
    /// `git` exited with an error.
    #[error("Git exited with an error")]
    Exit,
    /// `git` output is not UTF-8.
    #[error("Git output is not UTF-8")]
    Utf8,
}
