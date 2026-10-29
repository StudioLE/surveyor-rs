//! Parsed CLI arguments registered in the service container.

use crate::prelude::*;
use usage_rs::Cli as UsageCli;

/// Parsed CLI arguments.
#[derive(Debug, UsageCli)]
#[usage(bin = "surveyor", about, version, unknown_flags = "error")]
pub struct CliArgs {
    /// Set the log level.
    #[usage(long, value_enum)]
    pub log_level: Option<LogLevel>,
    /// Subcommand to execute.
    #[usage(subcommand)]
    pub command: Subcommand,
}

impl FromServices for CliArgs {
    type Error = Infallible;

    fn from_services(_: &ServiceProvider) -> Result<Self, Report<Self::Error>>
    where
        Self: Sized,
    {
        Ok(CliArgs::parse())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;
    use usage_rs::test::{Page, argv, help, parse};

    /// [`CliArgs::parse_from`] binds a log level and the version flags.
    #[test]
    fn cli_args_parse_from() {
        // Arrange
        let input = argv([
            "--log-level",
            "debug",
            "version",
            "--allow-dirty",
            "--dry-run",
        ]);
        let words = input.words();
        let spec = CliArgs::spec();
        // Act
        let cli = parse(spec, &words, CliArgs::parse_from).expect("the command line should parse");
        // Assert
        assert_eq!(cli.log_level, Some(LogLevel::Debug));
        let expected = VersionArgs {
            allow_dirty: true,
            dry_run: true,
        };
        assert_eq!(cli.command, Subcommand::Version(expected));
    }

    /// [`CliArgs::parse_from`] binds the notes subcommand.
    #[test]
    fn cli_args_parse_from_notes() {
        // Arrange
        let input = argv(["notes"]);
        let words = input.words();
        let spec = CliArgs::spec();
        // Act
        let cli = parse(spec, &words, CliArgs::parse_from).expect("the command line should parse");
        // Assert
        assert_eq!(cli.command, Subcommand::Notes);
    }

    /// [`CliArgs::parse_from`] rejects a log level outside the declared choices.
    #[test]
    fn cli_args_parse_from_invalid_log_level() {
        // Arrange
        let input = argv(["--log-level", "nonsense", "notes"]);
        let words = input.words();
        let spec = CliArgs::spec();
        // Act
        let message =
            parse(spec, &words, CliArgs::parse_from).expect_err("`nonsense` is not a log level");
        // Assert
        assert!(message.contains("invalid value 'nonsense'"), "{message}");
    }

    /// [`CliArgs::parse_from`] rejects a version flag passed to notes.
    #[test]
    fn cli_args_parse_from_unknown_flag() {
        // Arrange
        let input = argv(["notes", "--dry-run"]);
        let words = input.words();
        let spec = CliArgs::spec();
        // Act
        let message = parse(spec, &words, CliArgs::parse_from)
            .expect_err("`--dry-run` is not a flag of `notes`");
        // Assert
        assert!(message.contains("--dry-run"), "{message}");
    }

    /// The root help page.
    ///
    /// - Replaces the version with a placeholder, as CI sets it at build time
    #[test]
    fn cli_args_help() {
        let spec = CliArgs::spec();
        let output = help(spec, &[], Page::Long).replace(env!("CARGO_PKG_VERSION"), "x.y.z");
        assert_snapshot!(output);
    }

    /// The help page for the version subcommand.
    #[test]
    fn cli_args_help_version() {
        let spec = CliArgs::spec();
        let output = help(spec, &["version"], Page::Long);
        assert_snapshot!(output);
    }

    /// The help page for the notes subcommand.
    #[test]
    fn cli_args_help_notes() {
        let spec = CliArgs::spec();
        let output = help(spec, &["notes"], Page::Long);
        assert_snapshot!(output);
    }
}
