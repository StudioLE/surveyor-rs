//! Check for hex text.

/// Check whether text is hex.
pub trait IsHex {
    /// Is every character an ASCII hex digit of either case?
    ///
    /// - `true` for empty text
    fn is_hex(&self) -> bool;
}

impl IsHex for str {
    fn is_hex(&self) -> bool {
        self.bytes().all(|byte| byte.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`IsHex::is_hex`] accepts both cases and rejects other characters.
    #[test]
    fn is_hex_is_hex() {
        assert!("09afAF".is_hex());
        assert!(!"abcdefg".is_hex());
        assert!(!"ab cd".is_hex());
    }
}
