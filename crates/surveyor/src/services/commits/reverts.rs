//! Links between revert commits and the commits they revert.

use crate::prelude::*;
use regex::Regex;
use std::sync::LazyLock;

/// Runs of alphanumeric characters, so hex runs are bounded by anything else.
static ALPHANUMERIC_RUN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("[0-9A-Za-z]+").expect("pattern should compile"));

/// Reverted commits, each mapped to the revert commit that names it by hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reverts(BTreeMap<CommitHash, CommitHash>);

impl Reverts {
    /// Hash of the revert commit that reverts `hash`, if any.
    #[must_use]
    pub fn get(&self, hash: &CommitHash) -> Option<CommitHash> {
        self.0.get(hash).copied()
    }
}

/// Is the trimmed message prefixed with `revert`, case-insensitive?
fn is_revert(commit: &GitCommit) -> bool {
    commit
        .message
        .trim_start()
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("revert"))
}

/// Get every maximal hex run in `message` long enough to be a [`ShortCommitHash`].
///
/// - Logs at debug and skips runs too long to be a hash
fn get_hash_prefixes(message: &str) -> Vec<ShortCommitHash> {
    ALPHANUMERIC_RUN
        .find_iter(message)
        .map(|run| run.as_str())
        .filter(|run| run.len() >= ShortCommitHash::MIN_LENGTH && run.is_hex())
        .filter_map(|run| {
            ShortCommitHash::from_str(run)
                .inspect_err(|report| {
                    debug!(run, error = %report, "Ignoring hex run in revert message");
                })
                .ok()
        })
        .collect()
}

impl From<&[GitCommit]> for Reverts {
    /// Link each commit a revert names by hash to that revert.
    ///
    /// - Expects `commits` newest first, so a reverted revert links nothing
    fn from(commits: &[GitCommit]) -> Self {
        let mut reverted_by = BTreeMap::new();
        for revert in commits {
            if reverted_by.contains_key(&revert.hash) || !is_revert(revert) {
                continue;
            }
            let prefixes = get_hash_prefixes(&revert.message);
            for commit in commits {
                let named = prefixes
                    .iter()
                    .any(|prefix| commit.hash.starts_with(prefix));
                if named && commit.hash != revert.hash {
                    reverted_by.insert(commit.hash, revert.hash);
                }
            }
        }
        Self(reverted_by)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`Reverts::from`] links only reverted commits, to the hash of their revert.
    ///
    /// - [`GitCommit::fixture_revert`] names the reverted commit by full hash
    /// - [`GitCommit::fixture_revert_of_revert`] names the reverted revert by short hash
    #[test]
    fn reverts_from() {
        // Arrange
        let commits = GitCommit::fixture_history();
        // Act
        let reverts = Reverts::from(commits.as_slice());
        // Assert
        let expected = Reverts(BTreeMap::from([
            (
                CommitHash::fixture("ddddddd4"),
                CommitHash::fixture("eeeeeee5"),
            ),
            (
                CommitHash::fixture("6666666f"),
                CommitHash::fixture("7777777a"),
            ),
        ]));
        assert_eq!(reverts, expected);
    }

    /// [`get_hash_prefixes`] keeps only maximal hex runs that fit a short hash.
    #[test]
    fn get_hash_prefixes_runs() {
        // Arrange
        let long = "a".repeat(ShortCommitHash::MAX_LENGTH + 1);
        let message = format!("revert abc1234_ABCDEF0 abcdef1g abc123 {long}");
        // Act
        let prefixes = get_hash_prefixes(&message);
        // Assert
        let prefixes: Vec<&str> = prefixes.iter().map(ShortCommitHash::as_str).collect();
        assert_eq!(prefixes, ["abc1234", "abcdef0"]);
    }
}
