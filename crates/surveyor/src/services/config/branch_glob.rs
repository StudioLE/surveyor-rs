//! Glob matching branch names.

use crate::prelude::*;
use globset::{Glob, GlobBuilder, GlobMatcher};

/// Glob matching branch names, such as `release/*`.
///
/// - `*` does not cross `/`
/// - Deserializes from a string, failing if invalid
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct BranchGlob {
    glob: Glob,
    matcher: GlobMatcher,
}

impl BranchGlob {
    /// Text of the glob.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.glob.glob()
    }

    /// Does the glob match `branch`?
    #[must_use]
    pub fn is_match(&self, branch: &str) -> bool {
        self.matcher.is_match(branch)
    }
}

impl FromStr for BranchGlob {
    type Err = Report<BranchGlobError>;

    /// Parse and compile a branch glob.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let glob = GlobBuilder::new(text)
            .literal_separator(true)
            .build()
            .change_context(BranchGlobError::Invalid)
            .attach("glob", text)?;
        Ok(Self {
            matcher: glob.compile_matcher(),
            glob,
        })
    }
}

impl TryFrom<String> for BranchGlob {
    type Error = Report<BranchGlobError>;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::from_str(&text)
    }
}

impl Display for BranchGlob {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl PartialEq for BranchGlob {
    fn eq(&self, other: &Self) -> bool {
        self.glob == other.glob
    }
}

impl Eq for BranchGlob {}

#[cfg(test)]
impl BranchGlob {
    /// Create a [`BranchGlob`] by parsing `text`, such as `release/*`.
    pub(crate) fn fixture(text: &str) -> Self {
        Self::from_str(text).expect("fixture branch glob should parse")
    }
}

/// Errors returned when parsing a [`BranchGlob`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum BranchGlobError {
    /// The text is not a valid glob.
    #[error("Channel branch is not a valid glob")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`BranchGlob::is_match`] does not let `*` cross a `/`.
    #[test]
    fn branch_glob_is_match() {
        let glob = BranchGlob::fixture("release/*");
        assert!(glob.is_match("release/1.x"));
        assert!(!glob.is_match("release/1.x/hotfix"));
    }

    /// [`BranchGlob::from_str`] rejects an invalid glob and names it.
    #[test]
    fn branch_glob_from_str_invalid() {
        let report = BranchGlob::from_str("release/[a").expect_err("invalid glob should fail");
        assert_eq!(report.current_context(), &BranchGlobError::Invalid);
        let display = report.to_string();
        assert!(display.contains("release/[a"), "{display}");
    }
}
