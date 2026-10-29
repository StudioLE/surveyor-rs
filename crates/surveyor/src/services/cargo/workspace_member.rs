//! Member crate of a Cargo workspace.

use crate::prelude::*;

/// Member crate of a Cargo workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMember {
    /// Package name.
    pub name: String,
    /// Absolute path of the crate `Cargo.toml`.
    pub manifest_path: PathBuf,
    /// Absolute directory of the crate.
    pub directory: PathBuf,
    /// Does the crate inherit `version` from `[workspace.package]`?
    pub inherits_version: bool,
    /// Can the crate be published?
    ///
    /// - `false` if `publish = false`
    pub publish: bool,
    /// Names of the members this crate depends on as normal or build dependencies.
    pub dependencies: BTreeSet<String>,
    /// Names of the members this crate depends on as normal or build dependencies by `path` only.
    pub unversioned_dependencies: BTreeSet<String>,
}

#[cfg(test)]
impl WorkspaceMember {
    /// Create a publishable `alpha` [`WorkspaceMember`] in `/repo/crates/alpha` that inherits the version.
    pub(crate) fn fixture() -> Self {
        Self {
            name: "alpha".to_owned(),
            manifest_path: PathBuf::from("/repo/crates/alpha/Cargo.toml"),
            directory: PathBuf::from("/repo/crates/alpha"),
            inherits_version: true,
            publish: true,
            dependencies: BTreeSet::new(),
            unversioned_dependencies: BTreeSet::new(),
        }
    }
}
