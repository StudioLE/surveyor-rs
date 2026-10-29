//! Source of the release notes template.

use crate::prelude::*;
use std::fs;

/// Built-in release notes template.
const DEFAULT_TEMPLATE: &str = include_str!("release-notes.md.jinja");

/// Provide the release notes template from `surveyor.toml` or the built-in template.
#[derive(FromServices)]
pub struct TemplateProvider {
    config: Arc<Config>,
    working_directory: Arc<WorkingDirectory>,
}

impl TemplateProvider {
    /// Get the release notes template.
    ///
    /// - Reads the `template` from `surveyor.toml` relative to the working directory if set
    /// - Returns the built-in template otherwise
    pub fn get(&self) -> Result<String, Report<TemplateProviderError>> {
        let Some(relative) = &self.config.template else {
            return Ok(DEFAULT_TEMPLATE.to_owned());
        };
        let path = self.working_directory.path().join(relative);
        debug!(path = %path.display(), "Reading release notes template");
        fs::read_to_string(&path)
            .change_context(TemplateProviderError::Read)
            .attach_path(&path)
    }
}

/// Errors returned by [`TemplateProvider`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TemplateProviderError {
    /// Unable to read the configured template.
    #[error("Unable to read the release notes template")]
    Read,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// [`TemplateProvider::get`] returns the built-in template without a configured one.
    #[test]
    fn template_provider_get() {
        // Arrange
        let provider = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<TemplateProvider>();
        // Act
        let template = provider.get().expect("should get template");
        // Assert
        assert_eq!(template, DEFAULT_TEMPLATE);
    }

    /// [`TemplateProvider::get`] reads the configured template relative to the working directory.
    #[test]
    fn template_provider_get_custom() {
        // Arrange
        let directory = TempDir::new().expect("should create temp dir");
        fs::write(directory.path().join("notes.md.jinja"), "custom").expect("should write");
        let config = Config {
            template: Some(PathBuf::from("notes.md.jinja")),
            ..Config::default()
        };
        let provider = ServiceBuilder::new()
            .with_test_services()
            .with_instance(config)
            .with_instance(WorkingDirectory::new(directory.path().to_path_buf()))
            .build()
            .expect::<TemplateProvider>();
        // Act
        let template = provider.get().expect("should get template");
        // Assert
        assert_eq!(template, "custom");
    }

    /// [`TemplateProvider::get`] names a configured template it cannot read.
    #[test]
    fn template_provider_get_missing() {
        // Arrange
        let config = Config {
            template: Some(PathBuf::from("missing.md.jinja")),
            ..Config::default()
        };
        let provider = ServiceBuilder::new()
            .with_test_services()
            .with_instance(config)
            .build()
            .expect::<TemplateProvider>();
        // Act
        let report = provider.get().expect_err("missing template should fail");
        // Assert
        assert_eq!(report.current_context(), &TemplateProviderError::Read);
        let display = report.to_string();
        assert!(display.contains("missing.md.jinja"), "{display}");
    }
}
