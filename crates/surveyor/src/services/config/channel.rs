//! Release channel read from `surveyor.toml`.

use crate::prelude::*;

/// Release channel, selected when the current branch matches one of its globs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    /// Name used as the pre-release identifier, such as `alpha` in `1.2.0-alpha.3`.
    pub name: ChannelName,
    /// Branch globs that select this channel.
    pub branches: Vec<BranchGlob>,
    /// Stage of versions built on this channel.
    ///
    /// - [`Stage::Release`] or [`Stage::Prerelease`]
    pub stage: Stage,
}

#[cfg(test)]
impl Channel {
    /// Create the default `release` [`Channel`] on the `release` branch.
    pub(crate) fn fixture() -> Self {
        Self {
            name: ChannelName::fixture("release"),
            branches: vec![BranchGlob::fixture("release")],
            stage: Stage::Release,
        }
    }

    /// Create the default `alpha` pre-release [`Channel`] on the `main` branch.
    pub(crate) fn fixture_prerelease() -> Self {
        Self {
            name: ChannelName::fixture("alpha"),
            branches: vec![BranchGlob::fixture("main")],
            stage: Stage::Prerelease,
        }
    }
}
