//! Resolution of commit message types to the configured types.

use crate::prelude::*;

/// Resolve the type written in a commit message to a configured [`CommitType`].
#[derive(FromServices)]
pub struct CommitTypeResolver {
    config: Arc<Config>,
}

impl CommitTypeResolver {
    /// Resolve the type written in a commit message to a configured [`CommitType`].
    ///
    /// - Compares both sides lowercased with one trailing `s` stripped
    /// - Keeps `message_type` with [`Change::None`] if no type matches
    pub fn resolve(&self, message_type: &str) -> CommitType {
        let normalized = message_type.normalize_type();
        let types = &self.config.types;
        let lists = [
            (&types.minor, Change::Minor),
            (&types.patch, Change::Patch),
            (&types.none, Change::None),
        ];
        lists
            .into_iter()
            .find_map(|(names, change)| {
                names
                    .iter()
                    .find(|name| name.normalize_type() == normalized)
                    .map(|name| CommitType {
                        kind: name.clone(),
                        change,
                    })
            })
            .unwrap_or_else(|| CommitType {
                kind: message_type.to_owned(),
                change: Change::None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`CommitTypeResolver::resolve`] maps each configured list to its change.
    #[test]
    fn commit_type_resolver_resolve() {
        // Arrange
        let resolver = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<CommitTypeResolver>();
        // Act
        let changes: Vec<Change> = ["feat", "fix", "chore"]
            .into_iter()
            .map(|message_type| resolver.resolve(message_type).change)
            .collect();
        // Assert
        assert_eq!(changes, [Change::Minor, Change::Patch, Change::None]);
    }

    /// [`CommitTypeResolver::resolve`] matches types case-insensitively with one trailing `s` stripped.
    #[test]
    fn commit_type_resolver_resolve_loose() {
        // Arrange
        let resolver = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<CommitTypeResolver>();
        // Act
        let tests = resolver.resolve("Tests");
        let doc = resolver.resolve("doc");
        // Assert
        assert_eq!(
            tests,
            CommitType {
                kind: "test".to_owned(),
                change: Change::None,
            }
        );
        assert_eq!(
            doc,
            CommitType {
                kind: "docs".to_owned(),
                change: Change::Patch,
            }
        );
    }

    /// [`CommitTypeResolver::resolve`] keeps an unknown type and implies no change.
    #[test]
    fn commit_type_resolver_resolve_unknown() {
        // Arrange
        let resolver = ServiceBuilder::new()
            .with_test_services()
            .build()
            .expect::<CommitTypeResolver>();
        // Act
        let commit_type = resolver.resolve("wip");
        // Assert
        assert_eq!(
            commit_type,
            CommitType {
                kind: "wip".to_owned(),
                change: Change::None,
            }
        );
    }
}
