//! Versioning of a Cargo workspace.

use crate::prelude::*;

/// Version a Cargo workspace from the repo version.
#[derive(FromServices)]
pub struct CargoVersioner {
    workspace: Arc<dyn GetWorkspace>,
    crate_versions: Arc<CrateVersionEvaluator>,
    manifests: Arc<ManifestVersionWriter>,
    lock: Arc<LockVersionWriter>,
    tables: Arc<CrateSummaryRenderer>,
    summary: Arc<dyn WriteSummary>,
}

impl CargoVersioner {
    /// Select each crate's version, write them into `Cargo.toml` and `Cargo.lock`, and append them to the step summary.
    ///
    /// - Logs a warning for each publishable crate with a path-only internal dependency
    /// - Appends the step summary only if `GITHUB_STEP_SUMMARY` is set
    /// - Logs instead of writing any file if `dry_run`
    pub fn version(
        &self,
        repo: &RepoVersion,
        dry_run: bool,
    ) -> Result<(), Report<CargoVersionerError>> {
        let workspace = self
            .workspace
            .get_workspace()
            .change_context(CargoVersionerError::Workspace)?;
        warn_unversioned(&workspace);
        let crates = self
            .crate_versions
            .evaluate(&workspace, repo)
            .change_context(CargoVersionerError::CrateVersions)?;
        self.manifests
            .write(&workspace, &crates, &repo.version, dry_run)
            .change_context(CargoVersionerError::Manifests)?;
        self.lock
            .write(&workspace, &crates, dry_run)
            .change_context(CargoVersionerError::Lock)?;
        let summary = self.tables.render(&crates);
        if dry_run {
            info!(summary, "Skipping crate step summary for dry run");
            return Ok(());
        }
        self.summary
            .write_summary(&summary)
            .change_context(CargoVersionerError::Summary)
    }
}

/// Log a warning for each publishable member with a path-only internal dependency.
fn warn_unversioned(workspace: &Workspace) {
    for member in workspace.members.iter().filter(|member| member.publish) {
        for dependency in &member.unversioned_dependencies {
            warn!(
                name = member.name,
                dependency,
                "Publishable crate depends on a workspace crate by `path` without `version`, so it cannot be published"
            );
        }
    }
}

/// Errors returned by [`CargoVersioner`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CargoVersionerError {
    /// Unable to read the workspace.
    #[error("Unable to read the Cargo workspace")]
    Workspace,
    /// Unable to select the crate versions.
    #[error("Unable to select the crate versions")]
    CrateVersions,
    /// Unable to write the manifests.
    #[error("Unable to write the versions into the manifests")]
    Manifests,
    /// Unable to write `Cargo.lock`.
    #[error("Unable to write the versions into the lock file")]
    Lock,
    /// Unable to write the step summary.
    #[error("Unable to write the crate versions to the step summary")]
    Summary,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`CargoVersioner::version`] writes manifests, `Cargo.lock`, and the crate step summary.
    ///
    /// - Writes only `beta`, as `alpha` inherits the version from a root with no `[workspace]`
    #[test]
    fn cargo_versioner_version() {
        // Arrange
        let services = ServiceBuilder::new().with_test_services().build();
        let versioner = services.expect::<CargoVersioner>();
        let repo = RepoVersion {
            version: Version::fixture("2.0.0-alpha.1"),
            stage: Stage::Prerelease,
            channel: Some(ChannelName::fixture("alpha")),
            ..RepoVersion::fixture()
        };
        // Act
        versioner.version(&repo, false).expect("should version");
        // Assert
        let manifests = services.expect::<WriteManifestSpy>().get_manifests();
        let paths: Vec<&Path> = manifests.keys().map(PathBuf::as_path).collect();
        assert_eq!(paths, [Path::new("/repo/crates/beta/Cargo.toml")]);
        let beta = manifests.values().next().expect("beta should be written");
        assert!(beta.contains("version = \"2.0.0-alpha.1\""), "{beta}");
        let locks = services.expect::<WriteLockSpy>().get_locks();
        let paths: Vec<&Path> = locks.keys().map(PathBuf::as_path).collect();
        assert_eq!(paths, [Path::new("/repo/Cargo.lock")]);
        let lock = locks.values().next().expect("lock should be written");
        assert!(
            lock.contains("name = \"beta\"\nversion = \"2.0.0-alpha.1\""),
            "{lock}"
        );
        let summaries = services.expect::<WriteSummarySpy>().get_summaries();
        assert_eq!(summaries.len(), 1);
    }

    /// [`CargoVersioner::version`] writes nothing on a dry run.
    #[test]
    fn cargo_versioner_version_dry_run() {
        // Arrange
        let services = ServiceBuilder::new().with_test_services().build();
        let versioner = services.expect::<CargoVersioner>();
        // Act
        versioner
            .version(&RepoVersion::fixture(), true)
            .expect("dry run should succeed");
        // Assert
        let manifests = services.expect::<WriteManifestSpy>().get_manifests();
        let locks = services.expect::<WriteLockSpy>().get_locks();
        let summaries = services.expect::<WriteSummarySpy>().get_summaries();
        assert_eq!(manifests, BTreeMap::new());
        assert_eq!(locks, BTreeMap::new());
        assert_eq!(summaries, Vec::<String>::new());
    }
}
