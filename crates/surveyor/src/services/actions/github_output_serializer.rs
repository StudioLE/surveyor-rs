//! Serialization of GitHub Actions step outputs.

use crate::prelude::*;
use serde_json::Value;
use std::fmt::Write as _;

/// Serialize a flat value as GitHub Actions step outputs for `GITHUB_OUTPUT`.
#[derive(Default, FromServices)]
pub struct GitHubOutputSerializer;

impl GitHubOutputSerializer {
    /// Serialize each field of `value` as a `key=value` line, in key order.
    ///
    /// - Skips `null` fields, such as a `None` option
    /// - Writes strings without quotes, as GitHub reads the value raw
    ///
    /// # Errors
    ///
    /// - [`GitHubOutputSerializerError::NotObject`] if `value` is not a struct or map
    /// - [`GitHubOutputSerializerError::Nested`] if a field is an array or object
    /// - [`GitHubOutputSerializerError::Multiline`] if a field holds a newline
    #[expect(clippy::unused_self, reason = "stateless DI service")]
    pub fn serialize<T: Serialize>(
        &self,
        value: &T,
    ) -> Result<String, Report<GitHubOutputSerializerError>> {
        let value =
            serde_json::to_value(value).change_context(GitHubOutputSerializerError::Serialize)?;
        let Value::Object(fields) = value else {
            return Err(Report::new(GitHubOutputSerializerError::NotObject));
        };
        let mut outputs = String::new();
        for (key, value) in fields {
            let value = match value {
                Value::Null => continue,
                Value::String(text) => text,
                Value::Bool(_) | Value::Number(_) => value.to_string(),
                Value::Array(_) | Value::Object(_) => {
                    return Err(Report::new(GitHubOutputSerializerError::Nested).attach("key", key));
                }
            };
            if value.contains('\n') {
                return Err(Report::new(GitHubOutputSerializerError::Multiline).attach("key", key));
            }
            writeln!(outputs, "{key}={value}").expect("writing to a String should not fail");
        }
        Ok(outputs)
    }
}

/// Errors returned by [`GitHubOutputSerializer`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GitHubOutputSerializerError {
    /// Unable to serialize the value.
    #[error("Unable to serialize the step outputs")]
    Serialize,
    /// The value is not a struct or map.
    #[error("Step outputs must be a struct or map")]
    NotObject,
    /// A field is an array or object.
    #[error("Step output must be a string, number, or bool")]
    Nested,
    /// A field holds a newline.
    #[error("Step output must not hold a newline")]
    Multiline,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`GitHubOutputSerializer::serialize`] writes each field as a line in key order.
    #[test]
    fn github_output_serializer_serialize() {
        // Arrange
        let serializer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<GitHubOutputSerializer>();
        // Act
        let outputs = serializer
            .serialize(&RepoVersion::fixture())
            .expect("should serialize");
        // Assert
        assert_eq!(
            outputs,
            "change=major\nchannel=release\nstage=release\ntagged=false\nversion=2.0.0\n"
        );
    }

    /// [`GitHubOutputSerializer::serialize`] skips the channel in the development stage.
    #[test]
    fn github_output_serializer_serialize_development() {
        // Arrange
        let serializer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<GitHubOutputSerializer>();
        let repo = RepoVersion {
            stage: Stage::Development,
            channel: None,
            ..RepoVersion::fixture()
        };
        // Act
        let outputs = serializer.serialize(&repo).expect("should serialize");
        // Assert
        assert_eq!(
            outputs,
            "change=major\nstage=development\ntagged=false\nversion=2.0.0\n"
        );
    }

    /// [`GitHubOutputSerializer::serialize`] rejects an array field.
    #[test]
    fn github_output_serializer_serialize_nested() {
        // Arrange
        let serializer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<GitHubOutputSerializer>();
        let value = BTreeMap::from([("crates", vec!["alpha"])]);
        // Act
        let report = serializer.serialize(&value).expect_err("array should fail");
        // Assert
        assert_eq!(
            report.current_context(),
            &GitHubOutputSerializerError::Nested
        );
    }

    /// [`GitHubOutputSerializer::serialize`] rejects a value with a newline.
    #[test]
    fn github_output_serializer_serialize_multiline() {
        // Arrange
        let serializer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<GitHubOutputSerializer>();
        let value = BTreeMap::from([("notes", "one\ntwo")]);
        // Act
        let report = serializer
            .serialize(&value)
            .expect_err("newline should fail");
        // Assert
        assert_eq!(
            report.current_context(),
            &GitHubOutputSerializerError::Multiline
        );
    }

    /// [`GitHubOutputSerializer::serialize`] rejects a value that is not a struct or map.
    #[test]
    fn github_output_serializer_serialize_not_object() {
        // Arrange
        let serializer = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<GitHubOutputSerializer>();
        // Act
        let report = serializer
            .serialize(&"2.0.0")
            .expect_err("string should fail");
        // Assert
        assert_eq!(
            report.current_context(),
            &GitHubOutputSerializerError::NotObject
        );
    }
}
