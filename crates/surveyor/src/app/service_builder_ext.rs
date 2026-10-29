//! Extension trait for configuring a [`ServiceBuilder`] with application services.

use crate::prelude::*;

/// Register all application services on a [`ServiceBuilder`].
pub trait ServiceBuilderExt {
    /// Register all application services.
    fn with_app_services(self) -> Self;

    /// Register all application services with succeeding stubs and spies.
    ///
    /// - Repo at `/repo` on `main` with [`VersionTags::fixture`] and [`GitCommit::fixture_history`]
    /// - [`Workspace::fixture`] with no crate touched since a tag
    /// - Every manifest is `[package]` with `version = "0.0.0"`
    /// - `Cargo.lock` has `alpha` and `beta` at `0.0.0`
    /// - GitHub repo `StudioLE/surveyor-rs` matching only `ccccccc3` to user `ada`
    /// - Default [`Config`]
    /// - Spies record manifests, `Cargo.lock`, GitHub Actions outputs, step summaries, and stdout without writing any file
    #[cfg(test)]
    fn with_test_services(self) -> Self;
}

impl ServiceBuilderExt for ServiceBuilder {
    fn with_app_services(self) -> Self {
        self.with_logging(create_logger)
            .with_trait::<dyn CheckClean, GitAdapter>()
            .with_trait::<dyn GetCommitAuthor, GhAdapter>()
            .with_trait::<dyn GetCommits, GitAdapter>()
            .with_trait::<dyn GetCurrentBranch, GitAdapter>()
            .with_trait::<dyn GetLastCommitTouching, GitAdapter>()
            .with_trait::<dyn GetRepoName, GhAdapter>()
            .with_trait::<dyn GetShortHash, GitAdapter>()
            .with_trait::<dyn GetTagsContaining, GitAdapter>()
            .with_trait::<dyn GetVersionTags, GitAdapter>()
            .with_trait::<dyn GetWorkspace, CargoAdapter>()
            .with_trait::<dyn ReadLock, CargoAdapter>()
            .with_trait::<dyn ReadManifest, CargoAdapter>()
            .with_trait::<dyn WriteConsole, ConsoleWriter>()
            .with_trait::<dyn WriteLock, CargoAdapter>()
            .with_trait::<dyn WriteManifest, CargoAdapter>()
            .with_trait::<dyn WriteOutput, ActionsAdapter>()
            .with_trait::<dyn WriteSummary, ActionsAdapter>()
            .with_type::<ActionsEnvironment>()
            .with_type::<AuthorResolver>()
            .with_type::<CargoVersioner>()
            .with_type::<ChannelResolver>()
            .with_type::<CliArgs>()
            .with_type::<CommitParser>()
            .with_type::<CommitProvider>()
            .with_type::<CommitRangeProvider>()
            .with_type::<CommitTypeResolver>()
            .with_type::<Config>()
            .with_type::<CrateSummaryRenderer>()
            .with_type::<CrateVersionEvaluator>()
            .with_type::<GitHubOutputSerializer>()
            .with_type::<LockVersionWriter>()
            .with_type::<ManifestVersionWriter>()
            .with_type::<NotesContextFactory>()
            .with_type::<NotesHandler>()
            .with_type::<NotesRenderer>()
            .with_type::<RepoVersionEvaluator>()
            .with_type::<SubcommandHandler>()
            .with_type::<TemplateProvider>()
            .with_type::<VersionHandler>()
            .with_type::<VersionSummaryRenderer>()
            .with_type::<WorkingDirectory>()
    }

    #[cfg(test)]
    fn with_test_services(self) -> Self {
        self.with_app_services()
            .with_instance(ActionsEnvironment::default())
            .with_instance(Config::default())
            .with_instance(WorkingDirectory::new(PathBuf::from("/repo")))
            .with_trait::<dyn CheckClean, CheckCleanStub>()
            .with_trait::<dyn GetCommitAuthor, GetCommitAuthorSpy>()
            .with_trait::<dyn GetCommits, GetCommitsStub>()
            .with_trait::<dyn GetCurrentBranch, GetCurrentBranchStub>()
            .with_trait::<dyn GetLastCommitTouching, GetLastCommitTouchingSpy>()
            .with_trait::<dyn GetRepoName, GetRepoNameStub>()
            .with_trait::<dyn GetShortHash, GetShortHashStub>()
            .with_trait::<dyn GetTagsContaining, GetTagsContainingSpy>()
            .with_trait::<dyn GetVersionTags, GetVersionTagsStub>()
            .with_trait::<dyn GetWorkspace, GetWorkspaceStub>()
            .with_trait::<dyn ReadLock, ReadLockStub>()
            .with_trait::<dyn ReadManifest, ReadManifestStub>()
            .with_trait::<dyn WriteConsole, WriteConsoleSpy>()
            .with_trait::<dyn WriteLock, WriteLockSpy>()
            .with_trait::<dyn WriteManifest, WriteManifestSpy>()
            .with_trait::<dyn WriteOutput, WriteOutputSpy>()
            .with_trait::<dyn WriteSummary, WriteSummarySpy>()
    }
}

#[cfg(not(test))]
fn create_logger(services: &ServiceProvider) -> Result<Logger, Report<ResolveError>> {
    let args = services.get::<CliArgs>()?;
    let logger = LoggerBuilder::new()
        .with_level(args.log_level.unwrap_or_default())
        .with_target("studiole_di::service_provider", LogLevel::Debug)
        .with_target(
            "studiole_di::traits::service_provider_get_trait",
            LogLevel::Debug,
        )
        .build();
    Ok(logger)
}

#[cfg(test)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "signature required by with_logging"
)]
fn create_logger(_services: &ServiceProvider) -> Result<Logger, Report<ResolveError>> {
    let logger = LoggerBuilder::new().with_level(LogLevel::Trace).build();
    Ok(logger)
}
