//! Owner and name of a GitHub repo.

use crate::prelude::*;

/// Owner and name of a GitHub repo, such as `StudioLE/surveyor-rs`.
///
/// - Holds a non-empty owner and name separated by one `/`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoName(String);

impl FromStr for RepoName {
    type Err = Report<RepoNameError>;

    /// Parse `<owner>/<name>`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let valid = text.split_once('/').is_some_and(|(owner, name)| {
            !owner.is_empty() && !name.is_empty() && !name.contains('/')
        });
        if !valid {
            return Err(Report::new(RepoNameError::Invalid).attach("repo", text));
        }
        Ok(Self(text.to_owned()))
    }
}

impl Display for RepoName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
impl RepoName {
    /// Create the `StudioLE/surveyor-rs` [`RepoName`].
    pub(crate) fn fixture() -> Self {
        Self::from_str("StudioLE/surveyor-rs").expect("fixture repo name should parse")
    }
}

/// Errors returned when parsing a [`RepoName`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum RepoNameError {
    /// The text is not a non-empty owner and name separated by one `/`.
    #[error("GitHub repo must be `<owner>/<name>`")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`RepoName::from_str`] parses an owner and name.
    #[test]
    fn repo_name_from_str() {
        let repo = RepoName::from_str("StudioLE/surveyor-rs").expect("should parse");
        assert_eq!(repo.to_string(), "StudioLE/surveyor-rs");
    }

    /// [`RepoName::from_str`] rejects a missing owner or name, and a second `/`.
    #[test]
    fn repo_name_from_str_invalid() {
        for text in [
            "surveyor-rs",
            "/surveyor-rs",
            "StudioLE/",
            "StudioLE/surveyor/rs",
        ] {
            let report = RepoName::from_str(text).expect_err(text);
            assert_eq!(report.current_context(), &RepoNameError::Invalid, "{text}");
        }
    }
}
