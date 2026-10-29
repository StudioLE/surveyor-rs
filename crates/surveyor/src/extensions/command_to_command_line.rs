//! Display of a [`Command`] as a command line.

use std::ffi::OsStr;
use std::iter;
use std::process::Command;

/// Display a [`Command`] as a command line.
pub trait ToCommandLine {
    /// Program and arguments joined by spaces, lossily converted to UTF-8.
    fn to_command_line(&self) -> String;
}

impl ToCommandLine for Command {
    fn to_command_line(&self) -> String {
        iter::once(self.get_program())
            .chain(self.get_args())
            .map(OsStr::to_string_lossy)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`ToCommandLine::to_command_line`] joins the program and arguments.
    #[test]
    fn to_command_line_to_command_line() {
        let mut command = Command::new("git");
        command.args(["log", "-1"]);
        assert_eq!(command.to_command_line(), "git log -1");
    }
}
