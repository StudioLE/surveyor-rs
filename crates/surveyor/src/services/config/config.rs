//! Settings read from `surveyor.toml` and registered in the service container.

use crate::prelude::*;
use std::fs;

/// Name of the optional config file in the working directory.
const CONFIG_FILE: &str = "surveyor.toml";

/// Settings read from `surveyor.toml`.
///
/// - Each key missing from the file takes its default
/// - A present `[[channels]]` or `[types]` replaces that default entirely
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Path of the release notes template, relative to the working directory.
    ///
    /// Default: the built-in template
    pub template: Option<PathBuf>,
    /// Release channels, matched against the branch in listed order.
    ///
    /// Default: `alpha` on `main` as pre-release, `release` on `release` as release
    pub channels: Vec<Channel>,
    /// Commit types grouped by the version change they imply.
    ///
    /// Default: [`CommitTypes::default`]
    pub types: CommitTypes,
}

impl Config {
    /// Read `surveyor.toml` from `directory`, or return the defaults if it is missing.
    fn from_directory(directory: &Path) -> Result<Self, Report<ConfigError>> {
        let path = directory.join(CONFIG_FILE);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                debug!(path = %path.display(), "No config file, using defaults");
                return Ok(Self::default());
            }
            Err(error) => {
                return Err(Report::new(error)
                    .change_context(ConfigError::Read)
                    .attach_path(&path));
            }
        };
        Self::from_str(&text).attach_path(&path)
    }

    /// Fail if a channel has the development stage.
    fn validate_channels(&self) -> Result<(), Report<ConfigError>> {
        for channel in &self.channels {
            if channel.stage == Stage::Development {
                return Err(Report::new(ConfigError::ChannelStage).attach("channel", &channel.name));
            }
        }
        Ok(())
    }

    /// Fail if two configured types are equal after [`NormalizeType::normalize_type`].
    fn validate_types(&self) -> Result<(), Report<ConfigError>> {
        let mut seen: BTreeMap<String, &str> = BTreeMap::new();
        let types = self
            .types
            .minor
            .iter()
            .chain(&self.types.patch)
            .chain(&self.types.none);
        for name in types {
            if let Some(existing) = seen.insert(name.normalize_type(), name) {
                return Err(Report::new(ConfigError::TypeCollision)
                    .attach("type", existing)
                    .attach("type", name));
            }
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            template: None,
            channels: vec![
                Channel {
                    name: ChannelName::from_str("alpha").expect("`alpha` should be a channel name"),
                    branches: vec![BranchGlob::from_str("main").expect("`main` should be a glob")],
                    stage: Stage::Prerelease,
                },
                Channel {
                    name: ChannelName::from_str("release")
                        .expect("`release` should be a channel name"),
                    branches: vec![
                        BranchGlob::from_str("release").expect("`release` should be a glob"),
                    ],
                    stage: Stage::Release,
                },
            ],
            types: CommitTypes::default(),
        }
    }
}

impl FromStr for Config {
    type Err = Report<ConfigError>;

    /// Parse and validate `surveyor.toml` content.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(text).change_context(ConfigError::Parse)?;
        config.validate_channels()?;
        config.validate_types()?;
        Ok(config)
    }
}

impl FromServices for Config {
    type Error = ConfigError;

    fn from_services(services: &ServiceProvider) -> Result<Self, Report<Self::Error>>
    where
        Self: Sized,
    {
        let directory = services
            .get::<WorkingDirectory>()
            .change_context(ConfigError::WorkingDirectory)?;
        Self::from_directory(directory.path())
    }
}

/// Errors returned when loading [`Config`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ConfigError {
    /// Unable to resolve the working directory.
    #[error("Unable to resolve the working directory for `surveyor.toml`")]
    WorkingDirectory,
    /// Unable to read the config file.
    #[error("Unable to read `surveyor.toml`")]
    Read,
    /// The config file is not valid TOML, has unknown keys, or has an invalid channel name or glob.
    #[error("Unable to parse `surveyor.toml`")]
    Parse,
    /// A channel has the development stage.
    #[error("Channel stage must be `release` or `prerelease`")]
    ChannelStage,
    /// Two types normalize to the same name.
    #[error("Commit types collide after lowercasing and removing a trailing `s`")]
    TypeCollision,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// [`Config::from_str`] returns the defaults for an empty file.
    #[test]
    fn config_from_str_empty() {
        let config = Config::from_str("").expect("empty config should parse");
        assert_eq!(config, Config::default());
    }

    /// [`Config::from_str`] reads every key.
    #[test]
    fn config_from_str() {
        // Arrange
        let text = r#"
            template = "notes.md.jinja"

            [[channels]]
            name = "beta"
            branches = ["main", "release/*"]
            stage = "prerelease"

            [types]
            minor = ["feat"]
            patch = ["fix"]
            none = ["chore"]
        "#;
        // Act
        let config = Config::from_str(text).expect("config should parse");
        // Assert
        let expected = Config {
            template: Some(PathBuf::from("notes.md.jinja")),
            channels: vec![Channel {
                name: ChannelName::fixture("beta"),
                branches: vec![
                    BranchGlob::fixture("main"),
                    BranchGlob::fixture("release/*"),
                ],
                ..Channel::fixture_prerelease()
            }],
            types: CommitTypes {
                minor: vec!["feat".to_owned()],
                patch: vec!["fix".to_owned()],
                none: vec!["chore".to_owned()],
            },
        };
        assert_eq!(config, expected);
    }

    /// [`Config::from_str`] replaces the default channels but keeps the default types.
    #[test]
    fn config_from_str_channels_only() {
        // Arrange
        let text = r#"
            [[channels]]
            name = "beta"
            branches = ["main"]
            stage = "prerelease"
        "#;
        // Act
        let config = Config::from_str(text).expect("config should parse");
        // Assert
        let expected = vec![Channel {
            name: ChannelName::fixture("beta"),
            ..Channel::fixture_prerelease()
        }];
        assert_eq!(config.channels, expected);
        assert_eq!(config.types, CommitTypes::default());
    }

    /// [`Config::from_str`] replaces the default types entirely when `[types]` is present.
    #[test]
    fn config_from_str_types_only() {
        // Arrange
        let text = r#"
            [types]
            minor = ["feat", "perf"]
        "#;
        // Act
        let config = Config::from_str(text).expect("config should parse");
        // Assert
        assert_eq!(config.types.minor, ["feat", "perf"]);
        assert_eq!(config.types.patch, Vec::<String>::new());
        assert_eq!(config.types.none, Vec::<String>::new());
        assert_eq!(config.channels, Config::default().channels);
    }

    /// [`Config::from_str`] rejects an unknown key.
    #[test]
    fn config_from_str_unknown_key() {
        let report = Config::from_str("colour = true").expect_err("unknown key should fail");
        assert_eq!(report.current_context(), &ConfigError::Parse);
    }

    /// [`Config::from_str`] rejects an unknown key inside a channel.
    #[test]
    fn config_from_str_unknown_channel_key() {
        // Arrange
        let text = r#"
            [[channels]]
            name = "beta"
            branches = ["main"]
            stage = "prerelease"
            environment = "staging"
        "#;
        // Act
        let report = Config::from_str(text).expect_err("unknown key should fail");
        // Assert
        assert_eq!(report.current_context(), &ConfigError::Parse);
    }

    /// [`Config::from_str`] rejects an invalid branch glob and names it.
    #[test]
    fn config_from_str_invalid_glob() {
        // Arrange
        let text = r#"
            [[channels]]
            name = "beta"
            branches = ["release/[a"]
            stage = "prerelease"
        "#;
        // Act
        let report = Config::from_str(text).expect_err("invalid glob should fail");
        // Assert
        assert_eq!(report.current_context(), &ConfigError::Parse);
        let debug = format!("{report:?}");
        assert!(debug.contains("release/[a"), "{debug}");
    }

    /// [`Config::from_str`] rejects a channel name that is not a semver identifier.
    #[test]
    fn config_from_str_invalid_channel_name() {
        // Arrange
        let text = r#"
            [[channels]]
            name = "beta.1"
            branches = ["main"]
            stage = "prerelease"
        "#;
        // Act
        let report = Config::from_str(text).expect_err("invalid name should fail");
        // Assert
        assert_eq!(report.current_context(), &ConfigError::Parse);
        let debug = format!("{report:?}");
        assert!(debug.contains("beta.1"), "{debug}");
    }

    /// [`Config::from_str`] rejects an empty channel name.
    #[test]
    fn config_from_str_empty_channel_name() {
        // Arrange
        let text = r#"
            [[channels]]
            name = ""
            branches = ["main"]
            stage = "prerelease"
        "#;
        // Act
        let report = Config::from_str(text).expect_err("empty name should fail");
        // Assert
        assert_eq!(report.current_context(), &ConfigError::Parse);
    }

    /// [`Config::from_str`] rejects a channel with the development stage.
    #[test]
    fn config_from_str_development_stage() {
        // Arrange
        let text = r#"
            [[channels]]
            name = "dev"
            branches = ["main"]
            stage = "development"
        "#;
        // Act
        let report = Config::from_str(text).expect_err("development stage should fail");
        // Assert
        assert_eq!(report.current_context(), &ConfigError::ChannelStage);
    }

    /// [`Config::from_str`] rejects types that collide after normalization, across lists.
    #[test]
    fn config_from_str_type_collision() {
        // Arrange
        let text = r#"
            [types]
            patch = ["docs"]
            none = ["Doc"]
        "#;
        // Act
        let report = Config::from_str(text).expect_err("colliding types should fail");
        // Assert
        assert_eq!(report.current_context(), &ConfigError::TypeCollision);
        let display = report.to_string();
        assert!(display.contains("docs"), "{display}");
        assert!(display.contains("Doc"), "{display}");
    }

    /// [`Config::from_services`] returns the defaults when `surveyor.toml` is missing.
    #[test]
    fn config_from_services_missing() {
        // Arrange
        let directory = TempDir::new().expect("should create temp dir");
        let services = ServiceBuilder::new()
            .with_app_services()
            .with_instance(WorkingDirectory::new(directory.path().to_path_buf()))
            .build();
        // Act
        let config = services.get::<Config>().expect("config should resolve");
        // Assert
        assert_eq!(*config, Config::default());
    }

    /// [`Config::from_services`] reads `surveyor.toml` from the working directory.
    #[test]
    fn config_from_services() {
        // Arrange
        let directory = TempDir::new().expect("should create temp dir");
        fs::write(
            directory.path().join(CONFIG_FILE),
            "template = \"notes.md.jinja\"\n",
        )
        .expect("should write config");
        let services = ServiceBuilder::new()
            .with_app_services()
            .with_instance(WorkingDirectory::new(directory.path().to_path_buf()))
            .build();
        // Act
        let config = services.get::<Config>().expect("config should resolve");
        // Assert
        assert_eq!(config.template, Some(PathBuf::from("notes.md.jinja")));
    }

    /// [`Config::from_services`] names the file when it fails to parse.
    #[test]
    fn config_from_services_invalid() {
        // Arrange
        let directory = TempDir::new().expect("should create temp dir");
        fs::write(directory.path().join(CONFIG_FILE), "template = [").expect("should write");
        let services = ServiceBuilder::new()
            .with_app_services()
            .with_instance(WorkingDirectory::new(directory.path().to_path_buf()))
            .build();
        // Act
        let report = services
            .get::<Config>()
            .expect_err("invalid config should fail");
        // Assert
        let display = format!("{report:?}");
        assert!(display.contains(CONFIG_FILE), "{display}");
    }
}
