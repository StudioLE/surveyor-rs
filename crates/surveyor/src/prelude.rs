//! Common imports used throughout the crate.

pub use crate::app::Cli;
pub(crate) use crate::app::*;
pub(crate) use crate::commands::*;
#[cfg(feature = "testing")]
pub use crate::extensions::VersionFixture;
pub(crate) use crate::extensions::*;
pub(crate) use crate::schema::*;
pub use crate::schema::{Change, CommitHash, ShortCommitHash, Stage};
pub(crate) use crate::services::*;
pub use crate::services::{
    CargoAdapter, CheckClean, CheckCleanError, CrateVersion, CrateVersionEvaluator, GetCommits,
    GetCurrentBranch, GetCurrentBranchError, GetLastCommitTouching, GetShortHash,
    GetTagsContaining, GetVersionTags, GetWorkspace, GitAdapter, GitCommit, LockVersionWriter,
    ManifestVersionWriter, ReadLock, ReadManifest, RepoVersion, VersionTags, WorkingDirectory,
    Workspace, WorkspaceMember, WriteLock, WriteManifest,
};

pub(crate) use semver::Version;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::collections::{BTreeMap, BTreeSet, HashMap};
pub(crate) use std::convert::Infallible;
pub(crate) use std::error::Error;
pub(crate) use std::fmt::{self, Debug, Display, Formatter};
pub(crate) use std::io::{self, Write};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::str::FromStr;
pub(crate) use std::sync::{Arc, Mutex};
pub(crate) use strum::Display;
pub(crate) use studiole_di::prelude::*;
pub(crate) use studiole_logging::prelude::*;
pub(crate) use studiole_report::prelude::*;
pub(crate) use thiserror::Error;
pub(crate) use tracing::{debug, error, info, trace, warn};
pub(crate) use usage_rs::Args;
