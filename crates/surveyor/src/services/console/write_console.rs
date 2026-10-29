//! Trait for writing to the console.

use crate::prelude::*;

/// Write text to the console.
pub trait WriteConsole: Send + Sync {
    /// Write `text` as-is.
    fn write(&self, text: &str) -> Result<(), Report<WriteConsoleError>>;

    /// Write `line` followed by a newline.
    fn write_line(&self, line: &str) -> Result<(), Report<WriteConsoleError>> {
        self.write(&format!("{line}\n"))
    }
}

/// Errors returned by [`WriteConsole`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WriteConsoleError {
    /// Unable to write to stdout.
    #[error("Unable to write to stdout")]
    Write,
}
