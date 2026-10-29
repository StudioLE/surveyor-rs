//! Selection of the version for each workspace member.

use crate::prelude::*;

/// Select the version for each workspace member.
#[derive(FromServices)]
pub struct CrateVersionEvaluator {
    last_commit: Arc<dyn GetLastCommitTouching>,
    tags_containing: Arc<dyn GetTagsContaining>,
}

impl CrateVersionEvaluator {
    /// Get the version of each member, sorted by name.
    ///
    /// - Gives the repo version to members that inherit it
    /// - Gives the repo version to members changed since their last counted tag
    /// - Keeps the oldest counted tag containing the last change of an unchanged member, preferring full releases
    pub fn evaluate(
        &self,
        workspace: &Workspace,
        repo: &RepoVersion,
    ) -> Result<Vec<CrateVersion>, Report<CrateVersionEvaluatorError>> {
        let mut versions = Vec::new();
        for member in &workspace.members {
            let kept = if member.inherits_version {
                None
            } else {
                self.get_kept(workspace, member, repo)?
            };
            versions.push(CrateVersion {
                name: member.name.clone(),
                bumped: kept.is_none(),
                version: kept.unwrap_or_else(|| repo.version.clone()),
                publish: member.publish,
            });
        }
        versions.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(versions)
    }

    /// Get the version to keep if the member has not changed since a counted tag.
    ///
    /// - Prefers the oldest counted full release, then the oldest counted pre-release
    fn get_kept(
        &self,
        workspace: &Workspace,
        member: &WorkspaceMember,
        repo: &RepoVersion,
    ) -> Result<Option<Version>, Report<CrateVersionEvaluatorError>> {
        let watched = get_watched(workspace, member);
        let paths: Vec<PathBuf> = watched
            .iter()
            .map(|watched| watched.directory.clone())
            .collect();
        let excludes = get_excludes(workspace, &watched);
        let hash = self
            .last_commit
            .get_last_commit_touching(&paths, &excludes)
            .change_context(CrateVersionEvaluatorError::LastCommit)
            .attach("crate", &member.name)?;
        let Some(hash) = hash else {
            debug!(
                name = member.name,
                "No commit touches the crate, so it is bumped"
            );
            return Ok(None);
        };
        let tags = self
            .tags_containing
            .get_tags_containing(&hash)
            .change_context(CrateVersionEvaluatorError::Tags)
            .attach("crate", &member.name)?;
        let kept = tags
            .into_iter()
            .filter(|tag| is_counted(tag, repo))
            .min_by_key(|tag| (!tag.is_release(), tag.clone()));
        if let Some(version) = &kept {
            debug!(name = member.name, %version, %hash, "Crate is unchanged");
        } else {
            debug!(
                name = member.name,
                %hash, "Crate changed since its last release"
            );
        }
        Ok(kept)
    }
}

/// Get the member and every member it depends on directly or transitively.
fn get_watched<'a>(
    workspace: &'a Workspace,
    member: &'a WorkspaceMember,
) -> Vec<&'a WorkspaceMember> {
    let mut watched = vec![member];
    let mut pending: Vec<&String> = member.dependencies.iter().collect();
    while let Some(name) = pending.pop() {
        if watched.iter().any(|watched| &watched.name == name) {
            continue;
        }
        let Some(dependency) = workspace.members.iter().find(|other| &other.name == name) else {
            continue;
        };
        watched.push(dependency);
        pending.extend(dependency.dependencies.iter());
    }
    watched
}

/// Get the directories of unwatched members nested inside a watched member's directory.
fn get_excludes(workspace: &Workspace, watched: &[&WorkspaceMember]) -> Vec<PathBuf> {
    workspace
        .members
        .iter()
        .filter(|other| !watched.iter().any(|watched| watched.name == other.name))
        .map(|other| other.directory.clone())
        .filter(|directory| {
            watched.iter().any(|watched| {
                directory.starts_with(&watched.directory) && directory != &watched.directory
            })
        })
        .collect()
}

/// Does `tag` count as an earlier release of a crate when building `repo`?
///
/// - Full releases always count
/// - Tags of the current channel count in the pre-release stage
fn is_counted(tag: &Version, repo: &RepoVersion) -> bool {
    if tag.is_release() {
        return true;
    }
    let (Stage::Prerelease, Some(channel)) = (repo.stage, &repo.channel) else {
        return false;
    };
    channel.get_number(tag).is_some()
}

/// Errors returned by [`CrateVersionEvaluator`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CrateVersionEvaluatorError {
    /// Unable to get the last commit touching a crate.
    #[error("Unable to get the last commit touching the crate")]
    LastCommit,
    /// Unable to get the tags containing the last commit.
    #[error("Unable to get the tags containing the crate's last commit")]
    Tags,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`CrateVersionEvaluator::evaluate`] gives the repo version to a crate that inherits it.
    #[test]
    fn crate_version_evaluator_evaluate_inherits() {
        // Arrange
        let workspace = Workspace {
            members: vec![WorkspaceMember::fixture()],
            ..Workspace::fixture()
        };
        let services = ServiceBuilder::new().with_test_services().build();
        let evaluator = services.expect::<CrateVersionEvaluator>();
        let repo = RepoVersion::fixture();
        // Act
        let versions = evaluator
            .evaluate(&workspace, &repo)
            .expect("should get versions");
        // Assert
        let expected = vec![CrateVersion::fixture()];
        assert_eq!(versions, expected);
        let lookups = services.expect::<GetLastCommitTouchingSpy>().get_lookups();
        assert_eq!(lookups, BTreeMap::new());
    }

    /// [`CrateVersionEvaluator::evaluate`] keeps the oldest counted tag for an unchanged crate.
    #[test]
    fn crate_version_evaluator_evaluate_unchanged() {
        // Arrange
        let workspace = Workspace {
            members: vec![WorkspaceMember {
                inherits_version: false,
                ..WorkspaceMember::fixture()
            }],
            ..Workspace::fixture()
        };
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetLastCommitTouchingSpy {
                hash: Some(CommitHash::fixture("aaaaaaa")),
                ..GetLastCommitTouchingSpy::default()
            })
            .with_instance(GetTagsContainingSpy {
                tags: vec![
                    Version::fixture("1.1.0-alpha.1"),
                    Version::fixture("1.1.0"),
                    Version::fixture("1.2.0"),
                ],
                ..GetTagsContainingSpy::default()
            })
            .build()
            .expect::<CrateVersionEvaluator>();
        let repo = RepoVersion::fixture();
        // Act
        let versions = evaluator
            .evaluate(&workspace, &repo)
            .expect("should get versions");
        // Assert
        let expected = vec![CrateVersion {
            version: Version::fixture("1.1.0"),
            bumped: false,
            ..CrateVersion::fixture()
        }];
        assert_eq!(versions, expected);
    }

    /// [`CrateVersionEvaluator::evaluate`] counts tags of the current channel in the pre-release stage.
    #[test]
    fn crate_version_evaluator_evaluate_prerelease() {
        // Arrange
        let workspace = Workspace {
            members: vec![WorkspaceMember {
                inherits_version: false,
                ..WorkspaceMember::fixture()
            }],
            ..Workspace::fixture()
        };
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetLastCommitTouchingSpy {
                hash: Some(CommitHash::fixture("aaaaaaa")),
                ..GetLastCommitTouchingSpy::default()
            })
            .with_instance(GetTagsContainingSpy {
                tags: vec![
                    Version::fixture("1.1.0-alpha.1"),
                    Version::fixture("1.1.0-beta.1"),
                ],
                ..GetTagsContainingSpy::default()
            })
            .build()
            .expect::<CrateVersionEvaluator>();
        let repo = RepoVersion {
            stage: Stage::Prerelease,
            channel: Some(ChannelName::fixture("alpha")),
            ..RepoVersion::fixture()
        };
        // Act
        let versions = evaluator
            .evaluate(&workspace, &repo)
            .expect("should get versions");
        // Assert
        let expected = vec![CrateVersion {
            version: Version::fixture("1.1.0-alpha.1"),
            bumped: false,
            ..CrateVersion::fixture()
        }];
        assert_eq!(versions, expected);
    }

    /// [`CrateVersionEvaluator::evaluate`] in the pre-release stage when a full release also contains the last change.
    #[test]
    fn crate_version_evaluator_evaluate_prerelease_released() {
        // Arrange
        let workspace = Workspace {
            members: vec![WorkspaceMember {
                inherits_version: false,
                ..WorkspaceMember::fixture()
            }],
            ..Workspace::fixture()
        };
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetLastCommitTouchingSpy {
                hash: Some(CommitHash::fixture("aaaaaaa")),
                ..GetLastCommitTouchingSpy::default()
            })
            .with_instance(GetTagsContainingSpy {
                tags: vec![
                    Version::fixture("1.1.0-alpha.1"),
                    Version::fixture("1.1.0"),
                    Version::fixture("1.2.0"),
                ],
                ..GetTagsContainingSpy::default()
            })
            .build()
            .expect::<CrateVersionEvaluator>();
        let repo = RepoVersion {
            stage: Stage::Prerelease,
            channel: Some(ChannelName::fixture("alpha")),
            ..RepoVersion::fixture()
        };
        // Act
        let versions = evaluator
            .evaluate(&workspace, &repo)
            .expect("should get versions");
        // Assert
        let expected = vec![CrateVersion {
            version: Version::fixture("1.1.0"),
            bumped: false,
            ..CrateVersion::fixture()
        }];
        assert_eq!(versions, expected);
    }

    /// [`CrateVersionEvaluator::evaluate`] bumps a crate whose last change is in no counted tag.
    #[test]
    fn crate_version_evaluator_evaluate_changed() {
        // Arrange
        let workspace = Workspace {
            members: vec![WorkspaceMember {
                inherits_version: false,
                ..WorkspaceMember::fixture()
            }],
            ..Workspace::fixture()
        };
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetLastCommitTouchingSpy {
                hash: Some(CommitHash::fixture("aaaaaaa")),
                ..GetLastCommitTouchingSpy::default()
            })
            .with_instance(GetTagsContainingSpy {
                tags: vec![Version::fixture("1.1.0-alpha.1")],
                ..GetTagsContainingSpy::default()
            })
            .build()
            .expect::<CrateVersionEvaluator>();
        let repo = RepoVersion {
            stage: Stage::Development,
            channel: None,
            ..RepoVersion::fixture()
        };
        // Act
        let versions = evaluator
            .evaluate(&workspace, &repo)
            .expect("should get versions");
        // Assert
        let expected = vec![CrateVersion::fixture()];
        assert_eq!(versions, expected);
    }

    /// [`CrateVersionEvaluator::evaluate`] bumps a crate no commit touches.
    #[test]
    fn crate_version_evaluator_evaluate_untouched() {
        // Arrange
        let workspace = Workspace {
            members: vec![WorkspaceMember {
                inherits_version: false,
                ..WorkspaceMember::fixture()
            }],
            ..Workspace::fixture()
        };
        let services = ServiceBuilder::new().with_test_services().build();
        let evaluator = services.expect::<CrateVersionEvaluator>();
        let repo = RepoVersion::fixture();
        // Act
        let versions = evaluator
            .evaluate(&workspace, &repo)
            .expect("should get versions");
        // Assert
        let expected = vec![CrateVersion::fixture()];
        assert_eq!(versions, expected);
        let hashes = services.expect::<GetTagsContainingSpy>().get_hashes();
        assert_eq!(hashes, Vec::new());
    }

    /// [`CrateVersionEvaluator::evaluate`] watches transitive dependencies and excludes nested members.
    #[test]
    fn crate_version_evaluator_evaluate_watched() {
        // Arrange
        let workspace = Workspace {
            members: vec![
                WorkspaceMember {
                    name: "root".to_owned(),
                    manifest_path: PathBuf::from("/repo/Cargo.toml"),
                    directory: PathBuf::from("/repo"),
                    inherits_version: false,
                    dependencies: BTreeSet::from(["b".to_owned()]),
                    ..WorkspaceMember::fixture()
                },
                WorkspaceMember {
                    name: "b".to_owned(),
                    manifest_path: PathBuf::from("/repo/crates/b/Cargo.toml"),
                    directory: PathBuf::from("/repo/crates/b"),
                    inherits_version: false,
                    dependencies: BTreeSet::from(["c".to_owned()]),
                    ..WorkspaceMember::fixture()
                },
                WorkspaceMember {
                    name: "c".to_owned(),
                    manifest_path: PathBuf::from("/repo/crates/c/Cargo.toml"),
                    directory: PathBuf::from("/repo/crates/c"),
                    inherits_version: false,
                    ..WorkspaceMember::fixture()
                },
                WorkspaceMember {
                    name: "d".to_owned(),
                    manifest_path: PathBuf::from("/repo/crates/d/Cargo.toml"),
                    directory: PathBuf::from("/repo/crates/d"),
                    inherits_version: false,
                    ..WorkspaceMember::fixture()
                },
            ],
            ..Workspace::fixture()
        };
        let services = ServiceBuilder::new().with_test_services().build();
        let evaluator = services.expect::<CrateVersionEvaluator>();
        let repo = RepoVersion::fixture();
        // Act
        let versions = evaluator
            .evaluate(&workspace, &repo)
            .expect("should get versions");
        // Assert
        let names: Vec<&str> = versions
            .iter()
            .map(|version| version.name.as_str())
            .collect();
        assert_eq!(names, ["b", "c", "d", "root"]);
        let lookups = services.expect::<GetLastCommitTouchingSpy>().get_lookups();
        let root = vec![
            PathBuf::from("/repo"),
            PathBuf::from("/repo/crates/b"),
            PathBuf::from("/repo/crates/c"),
        ];
        let excludes = lookups.get(&root).expect("root should be looked up");
        assert_eq!(excludes, &[PathBuf::from("/repo/crates/d")]);
    }
}
