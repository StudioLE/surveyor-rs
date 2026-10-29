//! Attachment of a [`Response`] to an error report.

use crate::prelude::*;

/// Attach a [`Response`] to an error report as structured context.
pub trait AttachResponse {
    /// Attach stderr, stdout, and exit code from `response` as key-value pairs.
    fn attach_response(self, response: Response) -> Self;
}

impl<C: Error + Send + Sync + 'static> AttachResponse for Report<C> {
    fn attach_response(mut self, response: Response) -> Self {
        if let Some(message) = response.stderr {
            self = self.attach("stderr", message);
        }
        if let Some(message) = response.stdout {
            self = self.attach("stdout", message);
        }
        if let Some(code) = response.status.code() {
            self = self.attach("exit", code);
        }
        self
    }
}
