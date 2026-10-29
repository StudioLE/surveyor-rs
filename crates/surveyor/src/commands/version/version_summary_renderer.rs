//! Rendering of the repo version step summary.

use crate::prelude::*;

/// Render the repo version as a markdown table for the step summary.
#[derive(Default, FromServices)]
pub struct VersionSummaryRenderer;

impl VersionSummaryRenderer {
    /// Render a table of the repo version and how it was reached.
    ///
    /// - Omits the `Channel` column in the development stage
    #[must_use]
    #[expect(clippy::unused_self, reason = "stateless DI service")]
    pub fn render(&self, repo: &RepoVersion) -> String {
        let mut headers = vec!["Version", "Stage"];
        let mut values = vec![format!("`{}`", repo.version), repo.stage.to_string()];
        if let Some(channel) = &repo.channel {
            headers.push("Channel");
            values.push(channel.to_string());
        }
        headers.extend(["Change", "Tagged"]);
        values.extend([repo.change.to_string(), repo.tagged.to_string()]);
        format!(
            "| {} |\n|{}\n| {} |\n",
            headers.join(" | "),
            " --- |".repeat(headers.len()),
            values.join(" | ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;

    /// [`VersionSummaryRenderer::render`] renders every column on a channel.
    #[test]
    fn version_summary_renderer_render() {
        // Arrange
        let renderer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<VersionSummaryRenderer>();
        // Act
        let summary = renderer.render(&RepoVersion::fixture());
        // Assert
        assert_snapshot!(summary);
    }

    /// [`VersionSummaryRenderer::render`] omits the channel in the development stage.
    #[test]
    fn version_summary_renderer_render_development() {
        // Arrange
        let renderer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<VersionSummaryRenderer>();
        let repo = RepoVersion {
            stage: Stage::Development,
            channel: None,
            ..RepoVersion::fixture()
        };
        // Act
        let summary = renderer.render(&repo);
        // Assert
        assert_snapshot!(summary);
    }
}
