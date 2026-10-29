//! Commits grouped into release notes sections.

use crate::prelude::*;

/// Commits grouped into the sections of the default release notes template.
///
/// - Each commit appears in one section only
/// - Commits keep their input order within each section
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct SectionsContext {
    /// Breaking commits of any type.
    pub breaking: Vec<Commit>,
    /// Non-breaking `feat` commits.
    pub features: Vec<Commit>,
    /// Non-breaking `fix` commits.
    pub fixes: Vec<Commit>,
    /// Non-breaking `perf` commits.
    pub performance: Vec<Commit>,
    /// Every other commit, including non-conventional commits.
    pub maintenance: Vec<Commit>,
}

impl SectionsContext {
    /// Create a new [`SectionsContext`] by grouping `commits` into sections.
    ///
    /// - Matches types case-insensitively with one trailing `s` ignored
    pub fn from_commits(commits: &[Commit]) -> Self {
        let mut sections = Self::default();
        for commit in commits {
            let kind = commit.kind.as_deref().map(NormalizeType::normalize_type);
            match kind.as_deref() {
                _ if commit.breaking => sections.breaking.push(commit.clone()),
                Some("feat") => sections.features.push(commit.clone()),
                Some("fix") => sections.fixes.push(commit.clone()),
                Some("perf") => sections.performance.push(commit.clone()),
                _ => sections.maintenance.push(commit.clone()),
            }
        }
        sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;

    /// [`SectionsContext::from_commits`] puts breaking commits first and every other commit in maintenance.
    #[test]
    fn sections_context_from_commits() {
        // Arrange
        let commits = [
            Commit {
                conventional: false,
                kind: None,
                subject: "not conventional".to_owned(),
                ..Commit::fixture()
            },
            Commit {
                kind: Some("docs".to_owned()),
                subject: "explain flag".to_owned(),
                ..Commit::fixture()
            },
            Commit {
                kind: Some("fix".to_owned()),
                subject: "drop flag".to_owned(),
                breaking: true,
                ..Commit::fixture()
            },
            Commit {
                kind: Some("perf".to_owned()),
                subject: "cache tags".to_owned(),
                ..Commit::fixture()
            },
            Commit {
                kind: Some("fix".to_owned()),
                subject: "mend parser".to_owned(),
                ..Commit::fixture()
            },
            Commit {
                kind: Some("chore".to_owned()),
                subject: "bump deps".to_owned(),
                ..Commit::fixture()
            },
            Commit {
                subject: "add flag".to_owned(),
                ..Commit::fixture()
            },
        ];
        // Act
        let sections = SectionsContext::from_commits(&commits);
        // Assert
        assert_yaml_snapshot!(sections);
    }

    /// [`SectionsContext::from_commits`] with types configured in another spelling.
    #[test]
    fn sections_context_from_commits_spelling() {
        // Arrange
        let commits = [
            Commit {
                kind: Some("Feat".to_owned()),
                ..Commit::fixture()
            },
            Commit {
                kind: Some("Fix".to_owned()),
                ..Commit::fixture()
            },
            Commit {
                kind: Some("PERF".to_owned()),
                ..Commit::fixture()
            },
        ];
        // Act
        let sections = SectionsContext::from_commits(&commits);
        // Assert
        assert_eq!(sections.features.len(), 1);
        assert_eq!(sections.fixes.len(), 1);
        assert_eq!(sections.performance.len(), 1);
        assert_eq!(sections.maintenance.len(), 0);
    }
}
