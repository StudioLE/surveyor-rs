//! Rendering of release notes.

use crate::prelude::*;
use minijinja::Environment;
use minijinja::syntax::SyntaxConfig;
use minijinja::value::Serde;

/// Name the template is registered under, so it is not HTML escaped.
const TEMPLATE_NAME: &str = "release-notes.md";

/// Render release notes from commits with `minijinja`.
#[derive(FromServices)]
pub struct NotesRenderer {
    templates: Arc<TemplateProvider>,
    contexts: Arc<NotesContextFactory>,
}

impl NotesRenderer {
    /// Render the release notes for `commits`.
    ///
    /// - Uses the template from [`TemplateProvider::get`]
    /// - Uses the data from [`NotesContextFactory::create`]
    /// - Trims blocks and strips leading block whitespace
    pub fn render(&self, commits: &[Commit]) -> Result<String, Report<NotesRendererError>> {
        let template = self
            .templates
            .get()
            .change_context(NotesRendererError::GetTemplate)?;
        let mut environment = Environment::new();
        let syntax = SyntaxConfig::builder()
            .trim_blocks(true)
            .lstrip_blocks(true)
            .build()
            .change_context(NotesRendererError::Syntax)?;
        environment.set_syntax(syntax);
        environment
            .add_template(TEMPLATE_NAME, &template)
            .change_context(NotesRendererError::Template)?;
        let context = self.contexts.create(commits);
        environment
            .get_template(TEMPLATE_NAME)
            .change_context(NotesRendererError::Template)?
            .render(Serde(&context))
            .change_context(NotesRendererError::Render)
    }
}

/// Errors returned by [`NotesRenderer`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum NotesRendererError {
    /// Unable to get the template.
    #[error("Unable to get the release notes template")]
    GetTemplate,
    /// Unable to configure the template syntax.
    #[error("Unable to configure the release notes template syntax")]
    Syntax,
    /// The template is not valid.
    #[error("Unable to parse the release notes template")]
    Template,
    /// Unable to render the template.
    #[error("Unable to render the release notes")]
    Render,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;
    use std::fs;
    use tempfile::TempDir;

    /// [`NotesRenderer::render`] renders every section of the default template.
    #[test]
    fn notes_renderer_render() {
        // Arrange
        let services = ServiceBuilder::new().with_test_services().build();
        let renderer = services.expect::<NotesRenderer>();
        let commits = services
            .expect::<CommitProvider>()
            .get(None)
            .expect("should get commits");
        // Act
        let notes = renderer.render(&commits).expect("should render");
        // Assert
        assert_snapshot!(notes);
    }

    /// [`NotesRenderer::render`] omits empty sections.
    #[test]
    fn notes_renderer_render_fixes_only() {
        // Arrange
        let renderer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<NotesRenderer>();
        let commits = vec![Commit {
            kind: Some("fix".to_owned()),
            subject: "mend parser".to_owned(),
            change: Change::Patch,
            ..Commit::fixture()
        }];
        // Act
        let notes = renderer.render(&commits).expect("should render");
        // Assert
        assert_snapshot!(notes);
    }

    /// [`NotesRenderer::render`] uses the configured template.
    #[test]
    fn notes_renderer_render_custom_template() {
        // Arrange
        let directory = TempDir::new().expect("should create temp dir");
        fs::write(
            directory.path().join("notes.md.jinja"),
            "{% for commit in commits %}{{ commit.subject }} {{ commit.author.name }} <{{ commit.author.email }}>\n{% endfor %}",
        )
        .expect("should write template");
        let config = Config {
            template: Some(PathBuf::from("notes.md.jinja")),
            ..Config::default()
        };
        let renderer = ServiceBuilder::new()
            .with_test_services()
            .with_instance(config)
            .with_instance(WorkingDirectory::new(directory.path().to_path_buf()))
            .build()
            .expect::<NotesRenderer>();
        let commits = vec![Commit::fixture()];
        // Act
        let notes = renderer.render(&commits).expect("should render");
        // Assert
        assert_eq!(notes, "one Ada Lovelace <ada@example.com>\n");
    }
}
