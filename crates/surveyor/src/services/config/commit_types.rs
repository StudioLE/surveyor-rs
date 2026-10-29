//! Commit types grouped by the version change they imply.

use crate::prelude::*;

/// Commit types grouped by the version change they imply.
///
/// - A list missing from a present `[types]` table is empty, not the default
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CommitTypes {
    /// Types that imply a minor change.
    ///
    /// Default: `["feat"]`
    #[serde(default)]
    pub minor: Vec<String>,
    /// Types that imply a patch change.
    ///
    /// Default: `["fix", "perf", "deps", "build", "docs", "revert"]`
    #[serde(default)]
    pub patch: Vec<String>,
    /// Types that imply no change.
    ///
    /// Default: `["ci", "chore", "test", "style", "refactor"]`
    #[serde(default)]
    pub none: Vec<String>,
}

impl Default for CommitTypes {
    fn default() -> Self {
        Self {
            minor: vec!["feat".to_owned()],
            patch: vec![
                "fix".to_owned(),
                "perf".to_owned(),
                "deps".to_owned(),
                "build".to_owned(),
                "docs".to_owned(),
                "revert".to_owned(),
            ],
            none: vec![
                "ci".to_owned(),
                "chore".to_owned(),
                "test".to_owned(),
                "style".to_owned(),
                "refactor".to_owned(),
            ],
        }
    }
}
