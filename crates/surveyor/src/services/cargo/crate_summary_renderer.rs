//! Rendering of the crate versions step summary.

use crate::prelude::*;
use std::fmt::Write as _;

/// Render the crate versions as a markdown table for the step summary.
#[derive(Default, FromServices)]
pub struct CrateSummaryRenderer;

impl CrateSummaryRenderer {
    /// Render a table of every crate's version and whether it was bumped.
    ///
    /// - Starts with a blank line, so it follows the repo version table
    #[must_use]
    #[expect(clippy::unused_self, reason = "stateless DI service")]
    pub fn render(&self, crates: &[CrateVersion]) -> String {
        let mut markdown = String::from("\n| Crate | Version | Bumped |\n| --- | --- | --- |\n");
        for version in crates {
            let bumped = if version.bumped { "✅" } else { "➖" };
            writeln!(
                markdown,
                "| `{}` | `{}` | {bumped} |",
                version.name, version.version
            )
            .expect("writing to a String should not fail");
        }
        markdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;

    /// [`CrateSummaryRenderer::render`] renders a bumped and a kept crate.
    #[test]
    fn crate_summary_renderer_render() {
        // Arrange
        let renderer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<CrateSummaryRenderer>();
        let crates = vec![
            CrateVersion::fixture(),
            CrateVersion {
                name: "beta".to_owned(),
                version: Version::new(1, 0, 0),
                bumped: false,
                ..CrateVersion::fixture()
            },
        ];
        // Act
        let summary = renderer.render(&crates);
        // Assert
        assert_snapshot!(summary);
    }
}
