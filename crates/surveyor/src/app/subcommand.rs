//! Subcommand variants.

use crate::prelude::*;
use usage_rs::Subcommands as UsageSubcommands;

/// Available CLI subcommands.
#[derive(Clone, Debug, Eq, PartialEq, UsageSubcommands)]
pub enum Subcommand {
    /// Determine the version from git tags and commits, and write it into `Cargo.toml` and `Cargo.lock`.
    Version(VersionArgs),
    /// Render release notes for the commits since the last release.
    Notes,
}
