//! Lexical normalization of paths.

use crate::prelude::*;
use std::path::Component;

/// Normalize a path without touching the filesystem.
pub trait NormalizePath {
    /// Path with `.` removed and each `..` applied to the component before it.
    fn normalize(&self) -> PathBuf;
}

impl NormalizePath for Path {
    fn normalize(&self) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in self.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    normalized.pop();
                }
                other => normalized.push(other),
            }
        }
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`NormalizePath::normalize`] applies `.` and `..` lexically.
    #[test]
    fn normalize_path_normalize() {
        let path = Path::new("/repo/crates/a/./../b");
        assert_eq!(path.normalize(), PathBuf::from("/repo/crates/b"));
    }
}
