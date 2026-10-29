//! Output of `cargo metadata`.

use crate::prelude::*;

/// Output of `cargo metadata --no-deps --format-version 1`, keeping only the fields Surveyor reads.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Metadata {
    /// Workspace member packages.
    pub packages: Vec<MetadataPackage>,
    /// Package ids of the workspace members.
    pub workspace_members: Vec<String>,
    /// Directory of the root `Cargo.toml`.
    pub workspace_root: PathBuf,
}
