//! Package in the output of `cargo metadata`.

use crate::prelude::*;

/// Package in the output of `cargo metadata`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct MetadataPackage {
    /// Package name.
    pub name: String,
    /// Package id, matched against [`Metadata::workspace_members`].
    pub id: String,
    /// Absolute path of the package `Cargo.toml`.
    pub manifest_path: PathBuf,
    /// Registries the package may publish to.
    ///
    /// - `Some` and empty if `publish = false`
    /// - `None` if any registry is allowed
    pub publish: Option<Vec<String>>,
    /// Declared dependencies of every kind.
    pub dependencies: Vec<MetadataDependency>,
}
