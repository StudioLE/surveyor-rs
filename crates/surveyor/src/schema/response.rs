//! Captured child process output.

use crate::prelude::*;
use std::process::{ExitStatus, Output};

/// Captured output from a child process.
pub struct Response {
    /// Trimmed stderr, or `None` if empty.
    pub stderr: Option<String>,
    /// Trimmed stdout, or `None` if empty.
    pub stdout: Option<String>,
    /// Exit status of the process.
    pub status: ExitStatus,
}

impl From<Output> for Response {
    fn from(output: Output) -> Self {
        Self {
            stderr: output.stderr.to_trimmed_string(),
            stdout: output.stdout.to_trimmed_string(),
            status: output.status,
        }
    }
}
