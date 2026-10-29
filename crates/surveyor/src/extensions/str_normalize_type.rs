//! Normalization of conventional commit types for matching.

/// Normalize a conventional commit type so near spellings match.
pub trait NormalizeType {
    /// Lowercase form with one trailing `s` removed.
    ///
    /// - `Tests` and `test` both normalize to `test`
    /// - `docs` and `doc` both normalize to `doc`
    fn normalize_type(&self) -> String;
}

impl NormalizeType for str {
    fn normalize_type(&self) -> String {
        let lower = self.to_lowercase();
        match lower.strip_suffix('s') {
            Some(stripped) => stripped.to_owned(),
            None => lower,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`NormalizeType::normalize_type`] lowercases and strips one trailing `s`.
    #[test]
    fn normalize_type_normalize_type() {
        assert_eq!("Tests".normalize_type(), "test");
        assert_eq!("doc".normalize_type(), "doc");
        assert_eq!("docs".normalize_type(), "doc");
        assert_eq!("feat".normalize_type(), "feat");
        assert_eq!("ss".normalize_type(), "s");
    }
}
