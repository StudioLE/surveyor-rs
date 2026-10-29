//! Writing to the console.

mod console_writer;
mod write_console;
#[cfg(test)]
mod write_console_spy;

pub use console_writer::*;
pub use write_console::*;
#[cfg(test)]
pub use write_console_spy::*;
