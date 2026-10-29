//! [`WriteConsole`] spy.

use crate::prelude::*;

/// [`WriteConsole`] spy recording everything written.
#[derive(Default)]
pub struct WriteConsoleSpy {
    /// Text written, concatenated in order.
    pub output: Mutex<String>,
}

impl WriteConsoleSpy {
    /// Get the text written, concatenated in order.
    pub fn get_output(&self) -> String {
        self.output
            .lock()
            .expect("lock should not be poisoned")
            .clone()
    }
}

impl WriteConsole for WriteConsoleSpy {
    fn write(&self, text: &str) -> Result<(), Report<WriteConsoleError>> {
        self.output
            .lock()
            .expect("lock should not be poisoned")
            .push_str(text);
        Ok(())
    }
}

impl FromServices for WriteConsoleSpy {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
