//! Commit parsed as a conventional commit.

use crate::prelude::*;

/// Commit parsed as a conventional commit.
///
/// - Non-conventional commits keep their first line as the subject and the rest as the body
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Commit {
    /// Full commit hash.
    pub hash: CommitHash,
    /// Author of the commit.
    pub author: Author,
    /// Is the message a conventional commit?
    pub conventional: bool,
    /// Configured type name if matched, else the raw type.
    ///
    /// - `None` for non-conventional commits
    pub kind: Option<String>,
    /// Scope in parentheses after the type.
    pub scope: Option<String>,
    /// Description after the type, or the first line of a non-conventional message.
    pub subject: String,
    /// Body between the subject and the footers.
    pub body: Option<String>,
    /// Footers in message order.
    pub footers: Vec<Footer>,
    /// Is the commit marked with `!` or a `BREAKING CHANGE` footer?
    pub breaking: bool,
    /// Description of the breaking change.
    ///
    /// - The `BREAKING CHANGE` footer value, else the subject if marked with `!`
    pub breaking_description: Option<String>,
    /// Version change implied by this commit alone.
    pub change: Change,
    /// Hash of the revert commit that reverts this one.
    pub reverted_by: Option<CommitHash>,
}

impl From<&GitCommit> for Commit {
    /// Create a non-conventional [`Commit`] from a [`GitCommit`].
    ///
    /// - Takes the first line as the subject and the trimmed rest, if any, as the body
    /// - Implies no change
    fn from(commit: &GitCommit) -> Self {
        let (subject, body) = commit
            .message
            .split_once('\n')
            .unwrap_or((commit.message.as_str(), ""));
        let body = body.trim();
        Self {
            hash: commit.hash,
            author: Author {
                name: commit.author_name.clone(),
                email: commit.author_email.clone(),
                github: None,
            },
            conventional: false,
            kind: None,
            scope: None,
            subject: subject.to_owned(),
            body: (!body.is_empty()).then(|| body.to_owned()),
            footers: Vec::new(),
            breaking: false,
            breaking_description: None,
            change: Change::None,
            reverted_by: None,
        }
    }
}

#[cfg(test)]
impl Commit {
    /// Create a `feat: one` [`Commit`] by Ada Lovelace with hash `aaaaaaa1`.
    ///
    /// - Matches [`GitCommit::fixture`] parsed with the default config
    pub(crate) fn fixture() -> Self {
        Self {
            hash: CommitHash::fixture("aaaaaaa1"),
            author: Author {
                name: "Ada Lovelace".to_owned(),
                email: "ada@example.com".to_owned(),
                github: None,
            },
            conventional: true,
            kind: Some("feat".to_owned()),
            scope: None,
            subject: "one".to_owned(),
            body: None,
            footers: Vec::new(),
            breaking: false,
            breaking_description: None,
            change: Change::Minor,
            reverted_by: None,
        }
    }
}
