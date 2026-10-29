//! Footer of a conventional commit.

use crate::prelude::*;
use git_conventional::{Footer as ConventionalFooter, FooterSeparator};

/// Footer of a conventional commit, such as `Refs: #12`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Footer {
    /// Footer key, such as `Refs` or `BREAKING CHANGE`.
    pub key: String,
    /// Footer value.
    ///
    /// - Keeps the `#` of a `Closes #12` style footer as `#12`
    pub value: String,
}

impl From<&ConventionalFooter<'_>> for Footer {
    /// Create a [`Footer`] from a parsed conventional commit footer.
    ///
    /// - Prefixes the value with `#` for a `Closes #12` style footer
    fn from(footer: &ConventionalFooter<'_>) -> Self {
        Self {
            key: footer.token().as_str().to_owned(),
            value: match footer.separator() {
                FooterSeparator::Ref => format!("#{}", footer.value()),
                _ => footer.value().to_owned(),
            },
        }
    }
}
