//! Release stage of a version.

use crate::prelude::*;

/// Release stage of a version.
#[derive(Clone, Copy, Debug, Deserialize, Display, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Stage {
    /// Full release such as `1.2.0`.
    Release,
    /// Pre-release on a channel such as `1.2.0-alpha.3`.
    Prerelease,
    /// Build from a branch with no channel such as `1.2.0-DEV+abcdef1`.
    Development,
}
