//! GitHub Actions environment registered in the service container.

use crate::prelude::*;
use std::env;
use std::ffi::OsString;

/// GitHub Actions environment variables, read once.
///
/// - Each field is `None` outside GitHub Actions or if the variable is empty
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionsEnvironment {
    /// Path of the step outputs file from `GITHUB_OUTPUT`.
    pub output: Option<PathBuf>,
    /// Path of the step summary file from `GITHUB_STEP_SUMMARY`.
    pub summary: Option<PathBuf>,
    /// Owner and repo from `GITHUB_REPOSITORY`, such as `StudioLE/surveyor-rs`.
    ///
    /// - `None` if the variable is not `<owner>/<name>`, logged as a warning
    pub repository: Option<RepoName>,
}

impl FromServices for ActionsEnvironment {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>>
    where
        Self: Sized,
    {
        Ok(Self {
            output: read_var("GITHUB_OUTPUT").map(PathBuf::from),
            summary: read_var("GITHUB_STEP_SUMMARY").map(PathBuf::from),
            repository: read_var("GITHUB_REPOSITORY").and_then(|repository| {
                RepoName::from_str(&repository.to_string_lossy())
                    .inspect_err(|report| {
                        warn!(error = %report, "Ignoring GITHUB_REPOSITORY");
                    })
                    .ok()
            }),
        })
    }
}

/// Read an environment variable, treating empty as unset.
fn read_var(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}
