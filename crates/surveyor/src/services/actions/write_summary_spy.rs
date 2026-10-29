//! [`WriteSummary`] spy.

use crate::prelude::*;

/// [`WriteSummary`] spy recording each summary.
#[derive(Default)]
pub struct WriteSummarySpy {
    /// Markdown of each summary written, in order.
    pub summaries: Mutex<Vec<String>>,
}

impl WriteSummarySpy {
    /// Get the markdown of each summary written, in order.
    pub fn get_summaries(&self) -> Vec<String> {
        self.summaries
            .lock()
            .expect("lock should not be poisoned")
            .clone()
    }
}

impl WriteSummary for WriteSummarySpy {
    fn write_summary(&self, markdown: &str) -> Result<(), Report<WriteSummaryError>> {
        self.summaries
            .lock()
            .expect("lock should not be poisoned")
            .push(markdown.to_owned());
        Ok(())
    }
}

impl FromServices for WriteSummarySpy {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
