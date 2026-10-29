//! Current working directory registered in the service container.

use crate::prelude::*;
use std::env;

/// Directory Surveyor runs from, holding the repo and `surveyor.toml`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkingDirectory(PathBuf);

impl WorkingDirectory {
    /// Create a [`WorkingDirectory`] from `path`.
    ///
    /// - Lets tests point the adapters at a temporary repo
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Path of the directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl FromServices for WorkingDirectory {
    type Error = WorkingDirectoryError;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>>
    where
        Self: Sized,
    {
        let path = env::current_dir().change_context(WorkingDirectoryError::Read)?;
        Ok(Self(path))
    }
}

/// Errors returned when resolving [`WorkingDirectory`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum WorkingDirectoryError {
    /// Unable to read the current directory.
    #[error("Unable to read the current directory")]
    Read,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`WorkingDirectory::from_services`] resolves the current directory.
    #[test]
    fn working_directory_from_services() {
        // Arrange
        let services = ServiceBuilder::new().with_app_services().build();
        // Act
        let directory = services.expect::<WorkingDirectory>();
        // Assert
        let expected = env::current_dir().expect("should read the current directory");
        assert_eq!(directory.path(), expected);
    }
}
