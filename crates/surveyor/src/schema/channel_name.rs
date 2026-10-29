//! Name of a release channel.

use crate::prelude::*;

/// Name of a release channel, used as the pre-release identifier such as `alpha` in `1.2.0-alpha.3`.
///
/// - Holds only `0-9`, `A-Z`, `a-z`, and `-`, and not only digits, so it is a valid semver identifier
/// - Deserializes from a string, failing if invalid
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(try_from = "String")]
pub struct ChannelName(String);

impl ChannelName {
    /// Characters of the name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Number N if `version` is a `<channel>.N` pre-release of this channel.
    #[must_use]
    pub fn get_number(&self, version: &Version) -> Option<u64> {
        let number = version
            .pre
            .as_str()
            .strip_prefix(self.as_str())?
            .strip_prefix('.')?;
        u64::from_str(number).ok()
    }
}

impl FromStr for ChannelName {
    type Err = Report<ChannelNameError>;

    /// Parse a channel name.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let valid = text.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && !text.chars().all(|c| c.is_ascii_digit());
        if !valid {
            return Err(Report::new(ChannelNameError::Invalid).attach("channel", text));
        }
        Ok(Self(text.to_owned()))
    }
}

impl TryFrom<String> for ChannelName {
    type Error = Report<ChannelNameError>;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::from_str(&text)
    }
}

impl Display for ChannelName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
impl ChannelName {
    /// Create a [`ChannelName`] by parsing `text`, such as `alpha`.
    pub(crate) fn fixture(text: &str) -> Self {
        Self::from_str(text).expect("fixture channel name should parse")
    }
}

/// Errors returned when parsing a [`ChannelName`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ChannelNameError {
    /// The name is empty, holds a character that is not allowed, or holds only digits.
    #[error("Channel name must hold only `0-9`, `A-Z`, `a-z`, and `-`, and not only digits")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`ChannelName::from_str`] accepts letters, digits, and `-`.
    #[test]
    fn channel_name_from_str() {
        let name = ChannelName::from_str("beta-2").expect("should parse");
        assert_eq!(name.as_str(), "beta-2");
    }

    /// [`ChannelName::get_number`] reads N only from this channel's `<channel>.N` pre-releases.
    #[test]
    fn channel_name_get_number() {
        let name = ChannelName::fixture("alpha");
        assert_eq!(name.get_number(&Version::fixture("1.2.0-alpha.3")), Some(3));
        assert_eq!(name.get_number(&Version::fixture("1.2.0-alphabet.3")), None);
        assert_eq!(name.get_number(&Version::fixture("1.2.0-alpha.x")), None);
        assert_eq!(name.get_number(&Version::fixture("1.2.0-beta.3")), None);
        assert_eq!(name.get_number(&Version::fixture("1.2.0")), None);
    }

    /// [`ChannelName::from_str`] rejects an empty name, a `.`, and digits only.
    #[test]
    fn channel_name_from_str_invalid() {
        let empty = ChannelName::from_str("").expect_err("empty should fail");
        let dotted = ChannelName::from_str("beta.1").expect_err("`.` should fail");
        let numeric = ChannelName::from_str("01").expect_err("digits only should fail");
        assert_eq!(empty.current_context(), &ChannelNameError::Invalid);
        assert_eq!(dotted.current_context(), &ChannelNameError::Invalid);
        assert_eq!(numeric.current_context(), &ChannelNameError::Invalid);
    }
}
