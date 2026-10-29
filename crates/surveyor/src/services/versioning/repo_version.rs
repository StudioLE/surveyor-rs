//! Version of the repo.

use crate::prelude::*;

/// Version of the repo and how it was reached.
///
/// - Serializes as the GitHub Actions step outputs
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RepoVersion {
    /// Version to build.
    pub version: Version,
    /// Stage of the version.
    pub stage: Stage,
    /// Name of the matched channel.
    ///
    /// - `None` in the development stage
    pub channel: Option<ChannelName>,
    /// Highest change implied by the commits since the last full release.
    ///
    /// - [`Change::None`] even though the version is still bumped as a patch
    pub change: Change,
    /// Is the version already a tag on `HEAD`?
    pub tagged: bool,
}

#[cfg(test)]
impl RepoVersion {
    /// Create an untagged `2.0.0` [`RepoVersion`] on the `release` channel with a major change.
    pub(crate) fn fixture() -> Self {
        Self {
            version: Version::new(2, 0, 0),
            stage: Stage::Release,
            channel: Some(ChannelName::fixture("release")),
            change: Change::Major,
            tagged: false,
        }
    }
}
