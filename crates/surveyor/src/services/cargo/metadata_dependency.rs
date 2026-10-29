//! Dependency in the output of `cargo metadata`.

use crate::prelude::*;

/// Dependency in the output of `cargo metadata`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct MetadataDependency {
    /// Name of the depended on package.
    pub name: String,
    /// Version requirement.
    ///
    /// - `*` if the dependency has a `path` but no `version`
    pub req: String,
    /// Kind of dependency.
    ///
    /// - `None` for a normal dependency, else `dev` or `build`
    pub kind: Option<String>,
    /// Absolute directory of a path dependency.
    pub path: Option<PathBuf>,
}
