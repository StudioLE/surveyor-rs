//! [`WriteOutput`] spy.

use crate::prelude::*;

/// [`WriteOutput`] spy recording each write.
#[derive(Default)]
pub struct WriteOutputSpy {
    /// `key=value` lines of each write, in order.
    pub outputs: Mutex<Vec<String>>,
}

impl WriteOutputSpy {
    /// Get the `key=value` lines of each write, in order.
    pub fn get_outputs(&self) -> Vec<String> {
        self.outputs
            .lock()
            .expect("lock should not be poisoned")
            .clone()
    }
}

impl WriteOutput for WriteOutputSpy {
    fn write_output(&self, outputs: &str) -> Result<(), Report<WriteOutputError>> {
        self.outputs
            .lock()
            .expect("lock should not be poisoned")
            .push(outputs.to_owned());
        Ok(())
    }
}

impl FromServices for WriteOutputSpy {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
