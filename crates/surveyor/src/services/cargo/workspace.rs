//! Cargo workspace.

use crate::prelude::*;

/// Cargo workspace and its members.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    /// Absolute path of the root `Cargo.toml`.
    pub manifest_path: PathBuf,
    /// Workspace members sorted by name.
    pub members: Vec<WorkspaceMember>,
}

#[cfg(test)]
impl Workspace {
    /// Create a [`Workspace`] at `/repo` with two members.
    ///
    /// - `alpha` inherits the workspace version and can be published
    /// - `beta` has its own version and can't be published
    pub(crate) fn fixture() -> Self {
        Self {
            manifest_path: PathBuf::from("/repo/Cargo.toml"),
            members: vec![
                WorkspaceMember::fixture(),
                WorkspaceMember {
                    name: "beta".to_owned(),
                    manifest_path: PathBuf::from("/repo/crates/beta/Cargo.toml"),
                    directory: PathBuf::from("/repo/crates/beta"),
                    inherits_version: false,
                    publish: false,
                    ..WorkspaceMember::fixture()
                },
            ],
        }
    }

    /// Create a [`Workspace`] at `/repo` with `inherit`, `own`, and `path-only` members.
    ///
    /// - `own` has its own version, the others inherit the workspace version
    pub(crate) fn fixture_manifests() -> Self {
        Self {
            manifest_path: PathBuf::from("/repo/Cargo.toml"),
            members: ["inherit", "own", "path-only"]
                .into_iter()
                .map(|name| WorkspaceMember {
                    name: name.to_owned(),
                    manifest_path: PathBuf::from(format!("/repo/crates/{name}/Cargo.toml")),
                    directory: PathBuf::from(format!("/repo/crates/{name}")),
                    inherits_version: name != "own",
                    ..WorkspaceMember::fixture()
                })
                .collect(),
        }
    }
}
