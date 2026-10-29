//! Replacement of a TOML string value without losing its formatting.

use toml_edit::{Item, Value};

/// Replace a TOML value with a string, keeping the surrounding whitespace and comments.
pub trait SetString {
    /// Replace the value with `text`, keeping its decor.
    fn set_string(&mut self, text: &str);
}

impl SetString for Item {
    fn set_string(&mut self, text: &str) {
        let Some(value) = self.as_value_mut() else {
            *self = toml_edit::value(text);
            return;
        };
        let decor = value.decor().clone();
        *value = Value::from(text);
        *value.decor_mut() = decor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use toml_edit::DocumentMut;

    /// [`SetString::set_string`] keeps a trailing comment.
    #[test]
    fn set_string_set_string() {
        // Arrange
        let mut document =
            DocumentMut::from_str("version = \"0.0.0\" # set by CI\n").expect("should parse");
        // Act
        document
            .get_mut("version")
            .expect("version should exist")
            .set_string("1.2.3");
        // Assert
        assert_eq!(document.to_string(), "version = \"1.2.3\" # set by CI\n");
    }
}
