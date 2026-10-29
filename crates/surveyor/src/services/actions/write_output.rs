//! Writing of GitHub Actions step outputs.

use crate::prelude::*;

/// Write GitHub Actions step outputs.
pub trait WriteOutput: Send + Sync {
    /// Append `outputs` to `GITHUB_OUTPUT`.
    ///
    /// - Expects `key=value` lines from [`GitHubOutputSerializer::serialize`]
    /// - Does nothing if `GITHUB_OUTPUT` is not set
    fn write_output(&self, outputs: &str) -> Result<(), Report<WriteOutputError>>;
}

impl WriteOutput for ActionsAdapter {
    fn write_output(&self, outputs: &str) -> Result<(), Report<WriteOutputError>> {
        let Some(path) = &self.environment.output else {
            trace!("GITHUB_OUTPUT is not set, skipping outputs");
            return Ok(());
        };
        debug!(path = %path.display(), outputs, "Writing outputs");
        Self::append(path, outputs).change_context(WriteOutputError::Append)
    }
}

/// Errors returned by [`WriteOutput::write_output`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WriteOutputError {
    /// Unable to append to `GITHUB_OUTPUT`.
    #[error("Unable to write the step outputs to GITHUB_OUTPUT")]
    Append,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// [`ActionsAdapter::write_output`] appends to existing outputs.
    #[test]
    fn actions_adapter_write_output() {
        // Arrange
        let directory = TempDir::new().expect("should create temp dir");
        let path = directory.path().join("output");
        fs::write(&path, "existing=1\n").expect("should write");
        let environment = ActionsEnvironment {
            output: Some(path.clone()),
            ..ActionsEnvironment::default()
        };
        let adapter = ServiceBuilder::new()
            .with_test_services()
            .with_instance(environment)
            .build()
            .expect::<ActionsAdapter>();
        // Act
        adapter
            .write_output("stage=release\nversion=1.2.0\n")
            .expect("should write output");
        // Assert
        let text = fs::read_to_string(path).expect("should read");
        assert_eq!(text, "existing=1\nstage=release\nversion=1.2.0\n");
    }

    /// [`ActionsAdapter::write_output`] does nothing without `GITHUB_OUTPUT`.
    #[test]
    fn actions_adapter_write_output_unset() {
        // Arrange
        let adapter = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<ActionsAdapter>();
        // Act
        let output = adapter.write_output("version=1.2.0\n");
        // Assert
        output.expect("unset output should succeed");
    }

    /// [`ActionsAdapter::write_output`] names the file it cannot write.
    #[test]
    fn actions_adapter_write_output_missing_directory() {
        // Arrange
        let directory = TempDir::new().expect("should create temp dir");
        let path = directory.path().join("missing/output");
        let environment = ActionsEnvironment {
            output: Some(path),
            ..ActionsEnvironment::default()
        };
        let adapter = ServiceBuilder::new()
            .with_test_services()
            .with_instance(environment)
            .build()
            .expect::<ActionsAdapter>();
        // Act
        let report = adapter
            .write_output("version=1.2.0\n")
            .expect_err("missing directory should fail");
        // Assert
        let display = format!("{report:?}");
        assert!(display.contains("missing/output"), "{display}");
    }
}
