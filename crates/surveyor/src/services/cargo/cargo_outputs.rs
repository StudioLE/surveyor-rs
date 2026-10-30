//! GitHub Actions step outputs for a Cargo workspace.

use crate::prelude::*;

/// GitHub Actions step outputs for a Cargo workspace.
///
/// - Serializes as the GitHub Actions step outputs
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CargoOutputs {
    /// `cargo publish` package flags, e.g. `-p alpha -p beta`.
    ///
    /// - One `-p` per bumped crate that can be published, in the order given
    /// - Empty if no crate is bumped and publishable
    pub cargo_publish_args: String,
}

impl From<&[CrateVersion]> for CargoOutputs {
    fn from(crates: &[CrateVersion]) -> Self {
        let flags: Vec<String> = crates
            .iter()
            .filter(|version| version.bumped && version.publish)
            .map(|version| format!("-p {}", version.name))
            .collect();
        Self {
            cargo_publish_args: flags.join(" "),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`CargoOutputs::from`] flags only the crates that are bumped and publishable.
    #[test]
    fn cargo_outputs_from() {
        // Arrange
        let crates = vec![
            CrateVersion::fixture(),
            CrateVersion {
                name: "beta".to_owned(),
                publish: false,
                ..CrateVersion::fixture()
            },
            CrateVersion {
                name: "delta".to_owned(),
                ..CrateVersion::fixture()
            },
            CrateVersion {
                name: "gamma".to_owned(),
                bumped: false,
                ..CrateVersion::fixture()
            },
        ];
        // Act
        let outputs = CargoOutputs::from(crates.as_slice());
        // Assert
        assert_eq!(outputs.cargo_publish_args, "-p alpha -p delta");
    }

    /// [`CargoOutputs::from`] leaves the publish flags empty if no crate is bumped and publishable.
    #[test]
    fn cargo_outputs_from_none() {
        // Arrange
        let crates = vec![CrateVersion {
            bumped: false,
            ..CrateVersion::fixture()
        }];
        // Act
        let outputs = CargoOutputs::from(crates.as_slice());
        // Assert
        assert_eq!(outputs.cargo_publish_args, "");
    }
}
