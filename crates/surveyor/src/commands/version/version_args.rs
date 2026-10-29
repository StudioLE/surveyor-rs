//! Arguments for the version subcommand.

use crate::prelude::*;

/// Parsed arguments for the version subcommand.
#[derive(Args, Clone, Debug, Default, Eq, PartialEq)]
pub struct VersionArgs {
    /// Continue when the working tree has changes, and mark the version as dirty.
    #[usage(long)]
    pub allow_dirty: bool,
    /// Log what would be written instead of writing manifests, `Cargo.lock`, and GitHub Actions files.
    #[usage(long)]
    pub dry_run: bool,
}
