//! Commit read from `git log`.

use crate::prelude::*;

/// Commit read from `git log`, before conventional commit parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitCommit {
    /// Full commit hash.
    pub hash: CommitHash,
    /// Author name.
    pub author_name: String,
    /// Author email.
    pub author_email: String,
    /// Full commit message with trailing whitespace removed.
    pub message: String,
}

#[cfg(test)]
impl GitCommit {
    /// Create a `feat: one` [`GitCommit`] by Ada Lovelace with hash `aaaaaaa1`.
    pub(crate) fn fixture() -> Self {
        Self {
            hash: CommitHash::fixture("aaaaaaa1"),
            author_name: "Ada Lovelace".to_owned(),
            author_email: "ada@example.com".to_owned(),
            message: "feat: one".to_owned(),
        }
    }

    /// Create a `git revert` commit and the commit it reverts, newest first.
    ///
    /// - Names the reverted commit by its full hash, as `git revert` does
    pub(crate) fn fixture_revert() -> Vec<Self> {
        let reverted = CommitHash::fixture("ddddddd4");
        vec![
            Self {
                hash: CommitHash::fixture("eeeeeee5"),
                message: format!(
                    "Revert \"fix: mend parser\"\n\n\
                     This reverts commit {reverted}."
                ),
                ..Self::fixture()
            },
            Self {
                hash: reverted,
                message: "fix: mend parser".to_owned(),
                ..Self::fixture()
            },
        ]
    }

    /// Create a revert of a `git revert` commit, the revert, and the original, newest first.
    ///
    /// - Names the reverted revert by a 7 character short hash in a conventional message
    /// - Restores the original, so only the inner revert is reverted
    pub(crate) fn fixture_revert_of_revert() -> Vec<Self> {
        let original = CommitHash::fixture("5555555e");
        vec![
            Self {
                hash: CommitHash::fixture("7777777a"),
                message: "revert: undo 6666666".to_owned(),
                ..Self::fixture()
            },
            Self {
                hash: CommitHash::fixture("6666666f"),
                message: format!(
                    "Revert \"feat(cli): add settings flag\"\n\n\
                     This reverts commit {original}."
                ),
                ..Self::fixture()
            },
            Self {
                hash: original,
                message: "feat(cli): add settings flag".to_owned(),
                ..Self::fixture()
            },
        ]
    }

    /// Create a history that covers every release notes section, newest first.
    ///
    /// - Mixes types out of order to check grouping
    /// - Gives `ccccccc3` its own email so `AuthorResolver` looks it up separately
    /// - Ends with [`GitCommit::fixture_revert`] and [`GitCommit::fixture_revert_of_revert`]
    pub(crate) fn fixture_history() -> Vec<Self> {
        let mut history = vec![
            Self {
                hash: CommitHash::fixture("aaaaaaa1"),
                message: "feat(config): move settings\n\n\
                          Settings now live in `surveyor.toml`.\n\n\
                          BREAKING CHANGE: `--settings` is removed\n\
                          Refs: #3"
                    .to_owned(),
                ..Self::fixture()
            },
            Self {
                hash: CommitHash::fixture("bbbbbbb2"),
                message: "feat(cli): add flag\n\n\
                          First line.\n\n\
                          Second paragraph.\n\n\
                          Closes: #12"
                    .to_owned(),
                ..Self::fixture()
            },
            Self {
                hash: CommitHash::fixture("1111111a"),
                message: "chore(deps): bump serde".to_owned(),
                ..Self::fixture()
            },
            Self {
                hash: CommitHash::fixture("ccccccc3"),
                author_email: "ada@users.noreply.github.com".to_owned(),
                message: "feat: add notes".to_owned(),
                ..Self::fixture()
            },
            Self {
                hash: CommitHash::fixture("3333333c"),
                message: "Update readme".to_owned(),
                ..Self::fixture()
            },
            Self {
                hash: CommitHash::fixture("2222222b"),
                message: "ci: cache builds".to_owned(),
                ..Self::fixture()
            },
            Self {
                hash: CommitHash::fixture("fffffff6"),
                message: "perf(git): batch log calls".to_owned(),
                ..Self::fixture()
            },
            Self {
                hash: CommitHash::fixture("4444444d"),
                message: "docs: explain channels".to_owned(),
                ..Self::fixture()
            },
        ];
        history.extend(Self::fixture_revert());
        history.extend(Self::fixture_revert_of_revert());
        history
    }
}
