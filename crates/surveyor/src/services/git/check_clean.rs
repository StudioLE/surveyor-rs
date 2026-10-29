//! Check that the working tree has no changes.

use crate::prelude::*;

/// Check that the working tree has no changes.
pub trait CheckClean: Send + Sync {
    /// Fail if `git status --porcelain` lists any change, including untracked files.
    ///
    /// - Attaches each changed path
    fn check_clean(&self) -> Result<(), Report<CheckCleanError>>;
}

impl CheckClean for GitAdapter {
    fn check_clean(&self) -> Result<(), Report<CheckCleanError>> {
        let output = self
            .run(["status", "--porcelain", "--untracked-files=normal"])
            .change_context(CheckCleanError::Git)?;
        let paths = parse_status(&output);
        if paths.is_empty() {
            return Ok(());
        }
        let report = paths
            .into_iter()
            .fold(Report::new(CheckCleanError::Dirty), |report, path| {
                report.attach("path", path)
            });
        Err(report)
    }
}

/// Get the paths from `git status --porcelain` output.
fn parse_status(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.get(3..))
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Errors returned by [`CheckClean::check_clean`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CheckCleanError {
    /// Unable to run `git status`.
    #[error("Unable to check the working tree for changes")]
    Git,
    /// The working tree has changes.
    #[error("Working tree has changes; commit them or pass `--allow-dirty`")]
    Dirty,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`parse_status`] reads modified, untracked, and renamed paths.
    #[test]
    fn parse_status_porcelain() {
        // Arrange
        let output = " M Cargo.toml\n?? notes.md\nR  old.rs -> new.rs\n";
        // Act
        let paths = parse_status(output);
        // Assert
        assert_eq!(paths, ["Cargo.toml", "notes.md", "old.rs -> new.rs"]);
    }
}
