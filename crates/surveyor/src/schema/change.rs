//! Version change implied by commits.

use crate::prelude::*;

/// Version change implied by commits, ordered from least to most significant.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Display, Eq, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Change {
    /// No release needed, such as `chore` or `ci`.
    #[default]
    None,
    /// Bug fix, such as `fix`.
    Patch,
    /// New feature, such as `feat`.
    Minor,
    /// Breaking change, marked with `!` or a `BREAKING CHANGE` footer.
    Major,
}
