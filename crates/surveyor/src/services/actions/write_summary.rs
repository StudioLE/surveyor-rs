//! Writing of the GitHub Actions step summary.

use crate::prelude::*;

/// Write the GitHub Actions step summary.
pub trait WriteSummary: Send + Sync {
    /// Append `markdown` to `GITHUB_STEP_SUMMARY`.
    ///
    /// - Does nothing if `GITHUB_STEP_SUMMARY` is not set
    fn write_summary(&self, markdown: &str) -> Result<(), Report<WriteSummaryError>>;
}

impl WriteSummary for ActionsAdapter {
    fn write_summary(&self, markdown: &str) -> Result<(), Report<WriteSummaryError>> {
        let Some(path) = &self.environment.summary else {
            trace!("GITHUB_STEP_SUMMARY is not set, skipping summary");
            return Ok(());
        };
        debug!(path = %path.display(), "Writing step summary");
        Self::append(path, markdown).change_context(WriteSummaryError::Append)
    }
}

/// Errors returned by [`WriteSummary::write_summary`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WriteSummaryError {
    /// Unable to append to `GITHUB_STEP_SUMMARY`.
    #[error("Unable to write the step summary to GITHUB_STEP_SUMMARY")]
    Append,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// [`ActionsAdapter::write_summary`] appends markdown.
    #[test]
    fn actions_adapter_write_summary() {
        // Arrange
        let directory = TempDir::new().expect("should create temp dir");
        let path = directory.path().join("summary");
        let environment = ActionsEnvironment {
            summary: Some(path.clone()),
            ..ActionsEnvironment::default()
        };
        let adapter = ServiceBuilder::new()
            .with_test_services()
            .with_instance(environment)
            .build()
            .expect::<ActionsAdapter>();
        // Act
        adapter.write_summary("# One\n").expect("should write");
        adapter.write_summary("# Two\n").expect("should write");
        // Assert
        let text = fs::read_to_string(path).expect("should read");
        assert_eq!(text, "# One\n# Two\n");
    }

    /// [`ActionsAdapter::write_summary`] does nothing without `GITHUB_STEP_SUMMARY`.
    #[test]
    fn actions_adapter_write_summary_unset() {
        // Arrange
        let adapter = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<ActionsAdapter>();
        // Act
        let output = adapter.write_summary("# One\n");
        // Assert
        output.expect("unset summary should succeed");
    }
}
