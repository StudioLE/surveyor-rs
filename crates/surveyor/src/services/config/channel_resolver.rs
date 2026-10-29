//! Selection of the release channel for a branch.

use crate::prelude::*;

/// Select the configured [`Channel`] for a branch.
#[derive(FromServices)]
pub struct ChannelResolver {
    config: Arc<Config>,
}

impl ChannelResolver {
    /// Get the first configured channel with a glob matching `branch`.
    ///
    /// - Returns `None` if no channel matches, meaning the development stage
    /// - Logs a warning if pre-release channel names are not in ascending ASCII order
    pub fn resolve(&self, branch: &str) -> Option<Channel> {
        self.warn_unordered();
        for channel in &self.config.channels {
            if let Some(glob) = channel.branches.iter().find(|glob| glob.is_match(branch)) {
                debug!(branch, channel = %channel.name, %glob, "Matched channel");
                return Some(channel.clone());
            }
        }
        debug!(branch, "No channel matches the branch");
        None
    }

    /// Log a warning if pre-release channel names are not in ascending ASCII order.
    ///
    /// - Semver orders pre-releases by identifier, so `beta.1` follows `alpha.9` but not `rc.1`
    fn warn_unordered(&self) {
        let names: Vec<&str> = self
            .config
            .channels
            .iter()
            .filter(|channel| channel.stage == Stage::Prerelease)
            .map(|channel| channel.name.as_str())
            .collect();
        if names.is_sorted() {
            return;
        }
        warn!(
            channels = names.join(", "),
            "Pre-release channel names are not in ascending ASCII order, so semver will not order their versions as listed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`ChannelResolver::resolve`] selects the default pre-release channel for `main`.
    #[test]
    fn channel_resolver_resolve() {
        // Arrange
        let resolver = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<ChannelResolver>();
        // Act
        let channel = resolver.resolve("main");
        // Assert
        assert_eq!(channel, Some(Channel::fixture_prerelease()));
    }

    /// [`ChannelResolver::resolve`] returns `None` for a branch matching no channel.
    #[test]
    fn channel_resolver_resolve_no_match() {
        // Arrange
        let resolver = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<ChannelResolver>();
        // Act
        let channel = resolver.resolve("feature/login");
        // Assert
        assert_eq!(channel, None);
    }

    /// [`ChannelResolver::resolve`] picks the first matching channel in listed order.
    #[test]
    fn channel_resolver_resolve_first_match() {
        // Arrange
        let config = Config {
            channels: vec![
                Channel {
                    name: ChannelName::fixture("beta"),
                    branches: vec![BranchGlob::fixture("release/*")],
                    ..Channel::fixture_prerelease()
                },
                Channel {
                    branches: vec![
                        BranchGlob::fixture("release/*"),
                        BranchGlob::fixture("main"),
                    ],
                    ..Channel::fixture()
                },
            ],
            ..Config::default()
        };
        let resolver = ServiceBuilder::new()
            .with_test_services()
            .with_instance(config)
            .build()
            .expect::<ChannelResolver>();
        // Act
        let channel = resolver.resolve("release/1.x");
        // Assert
        let name = channel.map(|channel| channel.name);
        assert_eq!(name, Some(ChannelName::fixture("beta")));
    }

    /// [`ChannelResolver::resolve`] still matches when pre-release names are out of order.
    #[test]
    fn channel_resolver_resolve_unordered() {
        // Arrange
        let config = Config {
            channels: vec![
                Channel {
                    name: ChannelName::fixture("rc"),
                    ..Channel::fixture_prerelease()
                },
                Channel {
                    name: ChannelName::fixture("beta"),
                    branches: vec![BranchGlob::fixture("beta")],
                    ..Channel::fixture_prerelease()
                },
            ],
            ..Config::default()
        };
        let resolver = ServiceBuilder::new()
            .with_test_services()
            .with_instance(config)
            .build()
            .expect::<ChannelResolver>();
        // Act
        let channel = resolver.resolve("beta");
        // Assert
        let name = channel.map(|channel| channel.name);
        assert_eq!(name, Some(ChannelName::fixture("beta")));
    }
}
