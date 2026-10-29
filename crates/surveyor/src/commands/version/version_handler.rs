//! Handler for the version subcommand.

use crate::prelude::*;

/// Execute the version subcommand.
#[derive(FromServices)]
pub struct VersionHandler {
    check_clean: Arc<dyn CheckClean>,
    branch: Arc<dyn GetCurrentBranch>,
    channels: Arc<ChannelResolver>,
    repo_versions: Arc<RepoVersionEvaluator>,
    console: Arc<dyn WriteConsole>,
    serializer: Arc<GitHubOutputSerializer>,
    tables: Arc<VersionSummaryRenderer>,
    output: Arc<dyn WriteOutput>,
    summary: Arc<dyn WriteSummary>,
    cargo: Arc<CargoVersioner>,
}

impl VersionHandler {
    /// Determine the version, print it to stdout, and write it into `Cargo.toml` and `Cargo.lock`.
    ///
    /// - Writes GitHub Actions outputs and a step summary if their files are set
    /// - Writes the outputs and repo summary before the Cargo files
    /// - Logs instead of writing any file if `--dry-run`
    pub fn run(&self, args: VersionArgs) -> Result<(), Report<VersionHandlerError>> {
        trace!(?args, "Versioning");
        let dirty = self.get_dirty(args.allow_dirty)?;
        let branch = self
            .branch
            .get_current_branch()
            .change_context(VersionHandlerError::Branch)?;
        let channel = self.channels.resolve(&branch);
        let repo = self
            .repo_versions
            .evaluate(channel.as_ref(), dirty)
            .change_context(VersionHandlerError::Version)?;
        self.console
            .write_line(&repo.version.to_string())
            .change_context(VersionHandlerError::Console)?;
        let outputs = self
            .serializer
            .serialize(&repo)
            .change_context(VersionHandlerError::Output)?;
        let summary = self.tables.render(&repo);
        if args.dry_run {
            info!(
                outputs,
                summary, "Skipping GitHub Actions files for dry run"
            );
        } else {
            self.output
                .write_output(&outputs)
                .change_context(VersionHandlerError::Output)?;
            self.summary
                .write_summary(&summary)
                .change_context(VersionHandlerError::Summary)?;
        }
        self.cargo
            .version(&repo, args.dry_run)
            .change_context(VersionHandlerError::Cargo)
    }

    /// Is the working tree dirty?
    ///
    /// - Fails if dirty and not `allow_dirty`
    /// - Logs a warning naming the changes if dirty and `allow_dirty`
    fn get_dirty(&self, allow_dirty: bool) -> Result<bool, Report<VersionHandlerError>> {
        match self.check_clean.check_clean() {
            Ok(()) => Ok(false),
            Err(report) if allow_dirty && report.current_context() == &CheckCleanError::Dirty => {
                warn!(changes = %report, "Working tree has changes, so the version is marked dirty");
                Ok(true)
            }
            Err(report) => Err(report.change_context(VersionHandlerError::Clean)),
        }
    }
}

/// Errors returned by [`VersionHandler`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum VersionHandlerError {
    /// The working tree has changes or could not be checked.
    #[error("Working tree is not clean")]
    Clean,
    /// Unable to get the current branch.
    #[error("Unable to get the branch to pick a channel from")]
    Branch,
    /// Unable to calculate the version.
    #[error("Unable to calculate the version")]
    Version,
    /// Unable to write the version to stdout.
    #[error("Unable to write the version to stdout")]
    Console,
    /// Unable to write a step output.
    #[error("Unable to write the GitHub Actions outputs")]
    Output,
    /// Unable to write the step summary.
    #[error("Unable to write the GitHub Actions step summary")]
    Summary,
    /// Unable to version the Cargo workspace.
    #[error("Unable to version the Cargo workspace")]
    Cargo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;

    /// [`VersionHandler::run`] writes stdout, outputs, the step summary, and the Cargo files.
    #[test]
    fn version_handler_run() {
        // Arrange
        let services = ServiceBuilder::new().with_test_services().build();
        let handler = services.expect::<VersionHandler>();
        // Act
        handler.run(VersionArgs::default()).expect("should version");
        // Assert
        let stdout = services.expect::<WriteConsoleSpy>().get_output();
        assert_eq!(stdout, "2.0.0-alpha.1\n");
        let output = services.expect::<WriteOutputSpy>().get_outputs().join("");
        let summaries = services.expect::<WriteSummarySpy>().get_summaries();
        let manifests = services.expect::<WriteManifestSpy>().get_manifests();
        assert_snapshot!("output", output);
        assert_eq!(summaries.len(), 2);
        assert_snapshot!("summary", summaries.join(""));
        assert_eq!(manifests.len(), 1);
    }

    /// [`VersionHandler::run`] fails on a dirty tree without writing anything.
    #[test]
    fn version_handler_run_dirty() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(CheckCleanStub {
                changes: vec!["new.txt".to_owned()],
            })
            .build();
        let handler = services.expect::<VersionHandler>();
        let args = VersionArgs {
            dry_run: true,
            ..VersionArgs::default()
        };
        // Act
        let report = handler.run(args).expect_err("dirty tree should fail");
        // Assert
        assert_eq!(report.current_context(), &VersionHandlerError::Clean);
        let debug = format!("{report:?}");
        assert!(debug.contains("new.txt"), "{debug}");
        let manifests = services.expect::<WriteManifestSpy>().get_manifests();
        assert_eq!(manifests, BTreeMap::new());
    }

    /// [`VersionHandler::run`] marks the version dirty with `--allow-dirty`.
    #[test]
    fn version_handler_run_allow_dirty() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(CheckCleanStub {
                changes: vec!["new.txt".to_owned()],
            })
            .build();
        let handler = services.expect::<VersionHandler>();
        let args = VersionArgs {
            allow_dirty: true,
            ..VersionArgs::default()
        };
        // Act
        handler
            .run(args)
            .expect("allowed dirty tree should version");
        // Assert
        let outputs = services.expect::<WriteOutputSpy>().get_outputs().join("");
        assert!(
            outputs.contains("version=2.0.0-alpha.1+dirty\n"),
            "{outputs}"
        );
    }

    /// [`VersionHandler::run`] writes nothing on a dry run.
    #[test]
    fn version_handler_run_dry_run() {
        // Arrange
        let services = ServiceBuilder::new().with_test_services().build();
        let handler = services.expect::<VersionHandler>();
        let args = VersionArgs {
            dry_run: true,
            ..VersionArgs::default()
        };
        // Act
        handler.run(args).expect("dry run should succeed");
        // Assert
        let manifests = services.expect::<WriteManifestSpy>().get_manifests();
        let locks = services.expect::<WriteLockSpy>().get_locks();
        let outputs = services.expect::<WriteOutputSpy>().get_outputs();
        let summaries = services.expect::<WriteSummarySpy>().get_summaries();
        assert_eq!(manifests, BTreeMap::new());
        assert_eq!(locks, BTreeMap::new());
        assert_eq!(outputs, Vec::<String>::new());
        assert_eq!(summaries, Vec::<String>::new());
    }

    /// [`VersionHandler::run`] omits the channel on a development branch.
    #[test]
    fn version_handler_run_development() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCurrentBranchStub {
                branch: "feature/login".to_owned(),
            })
            .build();
        let handler = services.expect::<VersionHandler>();
        // Act
        handler.run(VersionArgs::default()).expect("should version");
        // Assert
        let output = services.expect::<WriteOutputSpy>().get_outputs().join("");
        assert_snapshot!(output);
    }
}
