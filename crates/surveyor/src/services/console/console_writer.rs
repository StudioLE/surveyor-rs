//! Console writer for writing to stdout.

use crate::prelude::*;

/// [`WriteConsole`] to stdout.
///
/// - Returns [`WriteConsoleError::Write`] on a broken pipe, where `print!` would panic
#[derive(Default, FromServices)]
pub struct ConsoleWriter;

impl WriteConsole for ConsoleWriter {
    fn write(&self, text: &str) -> Result<(), Report<WriteConsoleError>> {
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(text.as_bytes())
            .change_context(WriteConsoleError::Write)?;
        stdout.flush().change_context(WriteConsoleError::Write)
    }
}
