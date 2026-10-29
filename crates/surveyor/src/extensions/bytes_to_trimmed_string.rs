//! Conversion of captured process bytes into trimmed text.

/// Convert captured process bytes into trimmed text.
pub trait ToTrimmedString {
    /// Lossy UTF-8 text with surrounding whitespace removed, or `None` if empty.
    fn to_trimmed_string(&self) -> Option<String>;
}

impl ToTrimmedString for [u8] {
    fn to_trimmed_string(&self) -> Option<String> {
        let text = String::from_utf8_lossy(self).trim().to_owned();
        if text.is_empty() { None } else { Some(text) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`ToTrimmedString::to_trimmed_string`] trims text and maps blank text to `None`.
    #[test]
    fn to_trimmed_string_to_trimmed_string() {
        assert_eq!(
            b"  fatal: oops\n".to_trimmed_string(),
            Some("fatal: oops".to_owned())
        );
        assert_eq!(b" \n".to_trimmed_string(), None);
    }
}
