//! Lookup of the commits since a version.

use crate::prelude::*;

/// Separator between fields of one commit in `git log` output.
const FIELD_SEPARATOR: char = '\u{1f}';

/// Separator between commits in `git log` output.
const RECORD_SEPARATOR: char = '\u{1e}';

/// Get the commits since a version.
pub trait GetCommits: Send + Sync {
    /// Get the non-merge commits in `v<base>..HEAD`, newest first.
    ///
    /// - Gets every commit reachable from `HEAD` if `base` is `None`
    fn get_commits(&self, base: Option<Version>)
    -> Result<Vec<GitCommit>, Report<GetCommitsError>>;
}

impl GetCommits for GitAdapter {
    fn get_commits(
        &self,
        base: Option<Version>,
    ) -> Result<Vec<GitCommit>, Report<GetCommitsError>> {
        let range = match base {
            Some(base) => format!("v{base}..HEAD"),
            None => "HEAD".to_owned(),
        };
        let output = self
            .run(["log", "--no-merges", &get_log_format(), &range, "--"])
            .change_context(GetCommitsError::Git)?;
        parse_commits(&output).attach("range", range)
    }
}

/// `git log` format argument read by [`parse_commits`].
///
/// - Writes the hash, author name, author email, and message of each commit
/// - Separates fields with [`FIELD_SEPARATOR`] and ends each commit with [`RECORD_SEPARATOR`]
fn get_log_format() -> String {
    format!(
        "--format=%H{FIELD_SEPARATOR}%an{FIELD_SEPARATOR}%ae{FIELD_SEPARATOR}%B{RECORD_SEPARATOR}"
    )
}

/// Parse `git log` output with unit separated fields and record separated commits.
fn parse_commits(output: &str) -> Result<Vec<GitCommit>, Report<GetCommitsError>> {
    output
        .split(RECORD_SEPARATOR)
        .map(|record| record.trim_start_matches('\n'))
        .filter(|record| !record.is_empty())
        .map(|record| {
            let mut fields = record.splitn(4, FIELD_SEPARATOR);
            let (Some(hash), Some(author_name), Some(author_email), Some(message)) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                return Err(Report::new(GetCommitsError::Parse));
            };
            Ok(GitCommit {
                hash: CommitHash::from_str(hash).change_context(GetCommitsError::Hash)?,
                author_name: author_name.to_owned(),
                author_email: author_email.to_owned(),
                message: message.trim_end().to_owned(),
            })
        })
        .collect()
}

/// Errors returned by [`GetCommits::get_commits`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetCommitsError {
    /// Unable to run `git log`.
    #[error("Unable to get commits")]
    Git,
    /// A `git log` record has too few fields.
    #[error("Unable to parse a commit from git log")]
    Parse,
    /// A `git log` record has an invalid commit hash.
    #[error("Unable to parse a commit hash from git log")]
    Hash,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`parse_commits`] splits records and fields, keeping multi-line messages.
    #[test]
    fn parse_commits_multiline() {
        // Arrange
        let aaa = CommitHash::fixture("aaa");
        let bbb = CommitHash::fixture("bbb");
        let output = format!(
            "{aaa}{FIELD_SEPARATOR}Ada{FIELD_SEPARATOR}ada@example.com{FIELD_SEPARATOR}feat: add\n\nBody line\n\nRefs: #1\n\n{RECORD_SEPARATOR}\n{bbb}{FIELD_SEPARATOR}Bob{FIELD_SEPARATOR}bob@example.com{FIELD_SEPARATOR}fix: mend\n\n{RECORD_SEPARATOR}\n"
        );
        // Act
        let commits = parse_commits(&output).expect("output should parse");
        // Assert
        let expected = vec![
            GitCommit {
                hash: aaa,
                author_name: "Ada".to_owned(),
                author_email: "ada@example.com".to_owned(),
                message: "feat: add\n\nBody line\n\nRefs: #1".to_owned(),
            },
            GitCommit {
                hash: bbb,
                author_name: "Bob".to_owned(),
                author_email: "bob@example.com".to_owned(),
                message: "fix: mend".to_owned(),
            },
        ];
        assert_eq!(commits, expected);
    }

    /// [`parse_commits`] returns nothing for empty output.
    #[test]
    fn parse_commits_empty() {
        let commits = parse_commits("").expect("empty output should parse");
        assert_eq!(commits, Vec::new());
    }

    /// [`parse_commits`] rejects a record with too few fields.
    #[test]
    fn parse_commits_truncated() {
        let aaa = CommitHash::fixture("aaa");
        let output = format!("{aaa}{FIELD_SEPARATOR}Ada{RECORD_SEPARATOR}");
        let report = parse_commits(&output).expect_err("should fail");
        assert_eq!(report.current_context(), &GetCommitsError::Parse);
    }

    /// [`parse_commits`] rejects a record with an invalid hash.
    #[test]
    fn parse_commits_invalid_hash() {
        let output = format!(
            "aaa{FIELD_SEPARATOR}Ada{FIELD_SEPARATOR}ada@example.com{FIELD_SEPARATOR}feat: add{RECORD_SEPARATOR}"
        );
        let report = parse_commits(&output).expect_err("should fail");
        assert_eq!(report.current_context(), &GetCommitsError::Hash);
    }
}
