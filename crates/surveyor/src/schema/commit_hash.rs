//! Full hash of a git commit.

use crate::prelude::*;
use serde::Serializer;

/// Full hash of a git commit, as printed by `git log --format=%H`.
///
/// - Holds [`CommitHash::SHA1_LENGTH`] or [`CommitHash::SHA256_LENGTH`] lowercase hex characters
/// - Parses either case
/// - Serializes as a string
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommitHash(HashDigits);

/// Hex characters of a [`CommitHash`], sized by the hash algorithm of its repo.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum HashDigits {
    /// SHA-1, the default hash algorithm.
    Sha1([u8; CommitHash::SHA1_LENGTH]),
    /// SHA-256, from `git init --object-format=sha256`.
    Sha256([u8; CommitHash::SHA256_LENGTH]),
}

impl CommitHash {
    /// Number of hex characters in a SHA-1 hash.
    pub const SHA1_LENGTH: usize = 40;

    /// Number of hex characters in a SHA-256 hash.
    pub const SHA256_LENGTH: usize = 64;

    /// Lowercase hex characters of the hash.
    #[must_use]
    pub fn as_str(&self) -> &str {
        let bytes: &[u8] = match &self.0 {
            HashDigits::Sha1(bytes) => bytes,
            HashDigits::Sha256(bytes) => bytes,
        };
        str::from_utf8(bytes).expect("hash should be ASCII hex")
    }

    /// Does the hash start with `prefix`?
    #[must_use]
    pub fn starts_with(&self, prefix: &ShortCommitHash) -> bool {
        self.as_str().starts_with(prefix.as_str())
    }
}

impl FromStr for CommitHash {
    type Err = Report<CommitHashError>;

    /// Parse a full hash of either case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if !text.is_hex() {
            return Err(Report::new(CommitHashError::NotHex).attach("hash", text));
        }
        let lowercase = text.to_ascii_lowercase();
        let bytes = lowercase.as_bytes();
        if let Ok(bytes) = <[u8; Self::SHA1_LENGTH]>::try_from(bytes) {
            return Ok(Self(HashDigits::Sha1(bytes)));
        }
        if let Ok(bytes) = <[u8; Self::SHA256_LENGTH]>::try_from(bytes) {
            return Ok(Self(HashDigits::Sha256(bytes)));
        }
        Err(Report::new(CommitHashError::Length).attach("hash", text))
    }
}

impl Display for CommitHash {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for CommitHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(test)]
impl CommitHash {
    /// Create a SHA-1 [`CommitHash`] by repeating `seed`.
    ///
    /// - `aaaaaaa1` repeats to `aaaaaaa1aaaaaaa1aaaaaaa1aaaaaaa1aaaaaaa1`
    pub(crate) fn fixture(seed: &str) -> Self {
        let hash: String = seed.chars().cycle().take(Self::SHA1_LENGTH).collect();
        Self::from_str(&hash).expect("fixture seed should be hex")
    }
}

/// Errors returned when parsing a [`CommitHash`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CommitHashError {
    /// The text holds a character that is not hex.
    #[error("Commit hash must hold only hex characters")]
    NotHex,
    /// The text is not the length of a SHA-1 or SHA-256 hash.
    #[error("Commit hash must be {sha1} or {sha256} characters", sha1 = CommitHash::SHA1_LENGTH, sha256 = CommitHash::SHA256_LENGTH)]
    Length,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`CommitHash::from_str`] parses SHA-1 and SHA-256 hashes, lowercasing them.
    #[test]
    fn commit_hash_from_str() {
        // Arrange
        let sha1 = "ABCDEF01".repeat(5);
        let sha256 = "abcdef01".repeat(8);
        // Act
        let sha1 = CommitHash::from_str(&sha1).expect("should parse");
        let sha256 = CommitHash::from_str(&sha256).expect("should parse");
        // Assert
        assert_eq!(sha1.as_str(), "abcdef01".repeat(5));
        assert_eq!(sha256.as_str(), "abcdef01".repeat(8));
    }

    /// [`CommitHash::from_str`] rejects text that is not hex or not a full hash length.
    #[test]
    fn commit_hash_from_str_invalid() {
        // Arrange
        let not_hex = "g".repeat(CommitHash::SHA1_LENGTH);
        let short = "a".repeat(CommitHash::SHA1_LENGTH - 1);
        let between = "a".repeat(CommitHash::SHA1_LENGTH + 1);
        // Act
        let not_hex = CommitHash::from_str(&not_hex).expect_err("should fail");
        let short = CommitHash::from_str(&short).expect_err("should fail");
        let between = CommitHash::from_str(&between).expect_err("should fail");
        // Assert
        assert_eq!(not_hex.current_context(), &CommitHashError::NotHex);
        assert_eq!(short.current_context(), &CommitHashError::Length);
        assert_eq!(between.current_context(), &CommitHashError::Length);
    }

    /// [`CommitHash::starts_with`] matches a short hash prefix.
    #[test]
    fn commit_hash_starts_with() {
        // Arrange
        let hash = CommitHash::fixture("aaaaaaa1");
        let prefix = ShortCommitHash::from_str("aaaaaaa1aa").expect("should parse");
        let other = ShortCommitHash::from_str("bbbbbbb2").expect("should parse");
        // Act
        let matched = hash.starts_with(&prefix);
        let unmatched = hash.starts_with(&other);
        // Assert
        assert!(matched);
        assert!(!unmatched);
    }
}
