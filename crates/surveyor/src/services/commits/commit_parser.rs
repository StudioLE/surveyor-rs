//! Parsing of git commits as conventional commits.

use crate::prelude::*;
use git_conventional::Commit as ConventionalCommit;

/// Parse git commits as conventional commits with the configured types.
#[derive(FromServices)]
pub struct CommitParser {
    resolver: Arc<CommitTypeResolver>,
}

impl CommitParser {
    /// Parse a [`GitCommit`] as a conventional commit, falling back to a plain message.
    pub fn parse(&self, commit: &GitCommit) -> Commit {
        match ConventionalCommit::parse(&commit.message) {
            Ok(conventional) => self.parse_conventional(commit, &conventional),
            Err(_) => Commit::from(commit),
        }
    }

    /// Build a [`Commit`] from a conventional message.
    ///
    /// - Implies [`Change::Major`] if breaking, else the change of the configured type
    fn parse_conventional(&self, commit: &GitCommit, conventional: &ConventionalCommit) -> Commit {
        let breaking = conventional.breaking();
        let commit_type = self.resolver.resolve(conventional.type_().as_str());
        Commit {
            conventional: true,
            kind: Some(commit_type.kind),
            scope: conventional.scope().map(|scope| scope.as_str().to_owned()),
            subject: conventional.description().to_owned(),
            body: conventional.body().map(ToOwned::to_owned),
            footers: conventional.footers().iter().map(Footer::from).collect(),
            breaking,
            breaking_description: conventional.breaking_description().map(ToOwned::to_owned),
            change: if breaking {
                Change::Major
            } else {
                commit_type.change
            },
            ..Commit::from(commit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;

    /// [`CommitParser::parse`] parses type, scope, body, and footers.
    #[test]
    fn commit_parser_parse() {
        // Arrange
        let parser = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<CommitParser>();
        let git_commit = GitCommit {
            message: "feat(cli): add flag\n\nExplains why.\n\nRefs: #12\nCloses #13".to_owned(),
            ..GitCommit::fixture()
        };
        // Act
        let commit = parser.parse(&git_commit);
        // Assert
        assert_yaml_snapshot!(commit);
    }

    /// [`CommitParser::parse`] treats `!` and a `BREAKING CHANGE` footer as major.
    #[test]
    fn commit_parser_parse_breaking() {
        // Arrange
        let parser = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<CommitParser>();
        let bang_commit = GitCommit {
            message: "fix!: drop old flag".to_owned(),
            ..GitCommit::fixture()
        };
        let footer_commit = GitCommit {
            message: "chore: tidy\n\nBREAKING CHANGE: config moved".to_owned(),
            ..GitCommit::fixture()
        };
        // Act
        let bang = parser.parse(&bang_commit);
        let footer = parser.parse(&footer_commit);
        // Assert
        assert_eq!(bang.change, Change::Major);
        assert_eq!(footer.change, Change::Major);
        assert_eq!(bang.breaking_description.as_deref(), Some("drop old flag"));
        assert_eq!(footer.breaking_description.as_deref(), Some("config moved"));
    }

    /// [`CommitParser::parse`] splits a non-conventional message into subject and body.
    #[test]
    fn commit_parser_parse_not_conventional() {
        // Arrange
        let parser = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<CommitParser>();
        let git_commit = GitCommit {
            message: "Update readme\n\nMore words.".to_owned(),
            ..GitCommit::fixture()
        };
        // Act
        let commit = parser.parse(&git_commit);
        // Assert
        assert_yaml_snapshot!(commit);
    }
}
