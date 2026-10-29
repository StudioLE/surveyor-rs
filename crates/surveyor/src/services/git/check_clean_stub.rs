//! [`CheckClean`] stub.

use crate::prelude::*;

/// [`CheckClean`] stub.
#[derive(Default)]
pub struct CheckCleanStub {
    /// Changed paths, each attached to the [`CheckCleanError::Dirty`] error.
    ///
    /// - Clean if empty
    ///
    /// Default: empty
    pub changes: Vec<String>,
}

impl CheckClean for CheckCleanStub {
    fn check_clean(&self) -> Result<(), Report<CheckCleanError>> {
        if self.changes.is_empty() {
            return Ok(());
        }
        let report = self
            .changes
            .iter()
            .fold(Report::new(CheckCleanError::Dirty), |report, path| {
                report.attach("path", path)
            });
        Err(report)
    }
}

impl FromServices for CheckCleanStub {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>> {
        Ok(Self::default())
    }
}
