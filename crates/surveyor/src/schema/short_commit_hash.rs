//! Abbreviated hash of a git commit.

use crate::prelude::*;

/// Abbreviated hash of a git commit, as printed by `git rev-parse --short`.
///
/// - Holds [`ShortCommitHash::MIN_LENGTH`] to [`ShortCommitHash::MAX_LENGTH`] lowercase hex characters
/// - Parses either case
/// - May not name a commit, as when read from a revert message
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortCommitHash(String);

impl ShortCommitHash {
    /// Fewest hex characters in a short hash.
    pub const MIN_LENGTH: usize = 7;

    /// Most hex characters in a short hash, the length of a SHA-256 hash.
    pub const MAX_LENGTH: usize = CommitHash::SHA256_LENGTH;

    /// Lowercase hex characters of the hash.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ShortCommitHash {
    type Err = Report<ShortCommitHashError>;

    /// Parse a short hash of either case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if !text.is_hex() {
            return Err(Report::new(ShortCommitHashError::NotHex).attach("hash", text));
        }
        if !(Self::MIN_LENGTH..=Self::MAX_LENGTH).contains(&text.len()) {
            return Err(Report::new(ShortCommitHashError::Length).attach("hash", text));
        }
        Ok(Self(text.to_ascii_lowercase()))
    }
}

impl Display for ShortCommitHash {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Errors returned when parsing a [`ShortCommitHash`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ShortCommitHashError {
    /// The text holds a character that is not hex.
    #[error("Short commit hash must hold only hex characters")]
    NotHex,
    /// The text is too short or too long.
    #[error("Short commit hash must be {min} to {max} characters", min = ShortCommitHash::MIN_LENGTH, max = ShortCommitHash::MAX_LENGTH)]
    Length,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`ShortCommitHash::from_str`] parses the shortest and longest lengths, lowercasing them.
    #[test]
    fn short_commit_hash_from_str() {
        // Arrange
        let shortest = "A".repeat(ShortCommitHash::MIN_LENGTH);
        let longest = "b".repeat(ShortCommitHash::MAX_LENGTH);
        // Act
        let shortest = ShortCommitHash::from_str(&shortest).expect("should parse");
        let longest = ShortCommitHash::from_str(&longest).expect("should parse");
        // Assert
        assert_eq!(shortest.as_str(), "a".repeat(ShortCommitHash::MIN_LENGTH));
        assert_eq!(longest.as_str(), "b".repeat(ShortCommitHash::MAX_LENGTH));
    }

    /// [`ShortCommitHash::from_str`] rejects text that is not hex, too short, or too long.
    #[test]
    fn short_commit_hash_from_str_invalid() {
        // Arrange
        let not_hex = "g".repeat(ShortCommitHash::MIN_LENGTH);
        let short = "a".repeat(ShortCommitHash::MIN_LENGTH - 1);
        let long = "a".repeat(ShortCommitHash::MAX_LENGTH + 1);
        // Act
        let not_hex = ShortCommitHash::from_str(&not_hex).expect_err("should fail");
        let short = ShortCommitHash::from_str(&short).expect_err("should fail");
        let long = ShortCommitHash::from_str(&long).expect_err("should fail");
        // Assert
        assert_eq!(not_hex.current_context(), &ShortCommitHashError::NotHex);
        assert_eq!(short.current_context(), &ShortCommitHashError::Length);
        assert_eq!(long.current_context(), &ShortCommitHashError::Length);
    }
}
