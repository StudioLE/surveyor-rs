//! Calculation of the repo version from tags and commits.

use crate::prelude::*;
use semver::{BuildMetadata, Prerelease};

/// Footer key that sets the target version, compared case-insensitively.
const RELEASE_AS: &str = "Release-As";

/// Pre-release identifier of development versions.
const DEVELOPMENT: &str = "DEV";

/// Build metadata marking a version built from a working tree with changes.
const DIRTY: &str = "dirty";

/// Calculate the repo version from tags and conventional commits.
#[derive(FromServices)]
pub struct RepoVersionEvaluator {
    ranges: Arc<CommitRangeProvider>,
    short_hash: Arc<dyn GetShortHash>,
}

impl RepoVersionEvaluator {
    /// Get the version for `channel`, or the development version if `channel` is `None`.
    ///
    /// - Appends `dirty` build metadata if `dirty`
    /// - Logs a warning if the version is already a tag on `HEAD`
    pub fn evaluate(
        &self,
        channel: Option<&Channel>,
        dirty: bool,
    ) -> Result<RepoVersion, Report<RepoVersionEvaluatorError>> {
        let range = self
            .ranges
            .get()
            .change_context(RepoVersionEvaluatorError::Range)?;
        let base = range.base.clone().unwrap_or(Version::new(0, 0, 0));
        let change = range
            .commits
            .iter()
            .map(|commit| commit.change)
            .max()
            .unwrap_or_default();
        let target = match get_release_as(&range, &base)? {
            Some(target) => target,
            None => bump(&base, change, &range.tags.all),
        };
        let mut repo = RepoVersion {
            version: target,
            stage: channel.map_or(Stage::Development, |channel| channel.stage),
            channel: channel.map(|channel| channel.name.clone()),
            change,
            tagged: false,
        };
        match channel {
            None => repo.version = self.get_development(&repo.version, dirty)?,
            Some(channel) if channel.stage == Stage::Release => {
                if let Some(tagged) = get_release_at_head(&range.tags) {
                    repo.version = tagged;
                    repo.tagged = true;
                }
            }
            Some(channel) => {
                if let Some(tagged) =
                    get_prerelease_at_head(&range.tags, &repo.version, &channel.name)
                {
                    repo.version = tagged;
                    repo.tagged = true;
                } else {
                    repo.version = get_next_prerelease(&range.tags, &repo.version, &channel.name)?;
                }
            }
        }
        if repo.tagged {
            warn!(
                tag = format!("v{}", repo.version),
                "Version is already a tag on HEAD"
            );
        }
        if dirty && repo.stage != Stage::Development {
            repo.version.build = BuildMetadata::new(DIRTY).expect("`dirty` should be valid");
        }
        Ok(repo)
    }

    /// Get the `<target>-DEV+<short hash>` development version.
    ///
    /// - Appends `-dirty` to the build metadata if `dirty`
    fn get_development(
        &self,
        target: &Version,
        dirty: bool,
    ) -> Result<Version, Report<RepoVersionEvaluatorError>> {
        let hash = self
            .short_hash
            .get_short_hash()
            .change_context(RepoVersionEvaluatorError::ShortHash)?;
        let text = if dirty {
            format!("{hash}-{DIRTY}")
        } else {
            hash.to_string()
        };
        let build = BuildMetadata::new(&text)
            .change_context(RepoVersionEvaluatorError::Build)
            .attach("build", text)?;
        Ok(Version {
            pre: Prerelease::new(DEVELOPMENT).expect("`DEV` should be valid"),
            build,
            ..target.clone()
        })
    }
}

/// Get the target set by the newest commit with a `Release-As` footer.
///
/// # Errors
///
/// - Value is not semver: [`RepoVersionEvaluatorError::ReleaseAsInvalid`]
/// - Value has a pre-release or build metadata: [`RepoVersionEvaluatorError::ReleaseAsNotCore`]
/// - Value is not above `base`: [`RepoVersionEvaluatorError::ReleaseAsNotAbove`]
/// - Value is already a release tag: [`RepoVersionEvaluatorError::ReleaseAsTagged`]
fn get_release_as(
    range: &CommitRange,
    base: &Version,
) -> Result<Option<Version>, Report<RepoVersionEvaluatorError>> {
    let footer = range.commits.iter().find_map(|commit| {
        commit
            .footers
            .iter()
            .find(|footer| footer.key.eq_ignore_ascii_case(RELEASE_AS))
            .map(|footer| (commit, footer))
    });
    let Some((commit, footer)) = footer else {
        return Ok(None);
    };
    let target = Version::from_str(footer.value.trim())
        .change_context(RepoVersionEvaluatorError::ReleaseAsInvalid)
        .attach("commit", commit.hash)
        .attach("value", &footer.value)?;
    if !target.pre.is_empty() || !target.build.is_empty() {
        return Err(Report::new(RepoVersionEvaluatorError::ReleaseAsNotCore)
            .attach("commit", commit.hash)
            .attach("value", &footer.value));
    }
    if target <= *base {
        return Err(Report::new(RepoVersionEvaluatorError::ReleaseAsNotAbove)
            .attach("commit", commit.hash)
            .attach("target", &target)
            .attach("base", base));
    }
    if range.tags.all.contains(&target) {
        return Err(Report::new(RepoVersionEvaluatorError::ReleaseAsTagged)
            .attach("commit", commit.hash)
            .attach("tag", format!("v{target}")));
    }
    debug!(commit = %commit.hash, %target, "Target set by Release-As");
    Ok(Some(target))
}

/// Bump `base` by `change`, skipping full releases already tagged on other commits.
///
/// - Treats [`Change::None`] as [`Change::Patch`]
/// - Treats [`Change::Major`] as [`Change::Minor`] below `1.0.0`
fn bump(base: &Version, change: Change, tags: &[Version]) -> Version {
    let change = change.max(Change::Patch);
    let position = if base.major == 0 && change == Change::Major {
        Change::Minor
    } else {
        change
    };
    let mut target = increment(base, position);
    while tags
        .iter()
        .any(|tag| tag.is_release() && tag.has_same_core(&target))
    {
        debug!(%target, "Target is already a release tag, bumping again");
        target = increment(&target, position);
    }
    target
}

/// Increment `version` at `position`, resetting lower positions.
fn increment(version: &Version, position: Change) -> Version {
    match position {
        Change::Major => Version::new(version.major + 1, 0, 0),
        Change::Minor => Version::new(version.major, version.minor + 1, 0),
        Change::None | Change::Patch => {
            Version::new(version.major, version.minor, version.patch + 1)
        }
    }
}

/// Newest full release tag on `HEAD`, if any.
fn get_release_at_head(tags: &VersionTags) -> Option<Version> {
    tags.at_head
        .iter()
        .filter(|version| version.is_release())
        .max()
        .cloned()
}

/// Newest `<target>-<channel>.N` tag on `HEAD`, if any.
fn get_prerelease_at_head(
    tags: &VersionTags,
    target: &Version,
    channel: &ChannelName,
) -> Option<Version> {
    tags.at_head
        .iter()
        .filter(|version| get_prerelease_number(version, target, channel).is_some())
        .max()
        .cloned()
}

/// Get the next `<target>-<channel>.N` pre-release.
///
/// - N is one above the highest `v<target>-<channel>.M` tag in the repo, starting at 1
fn get_next_prerelease(
    tags: &VersionTags,
    target: &Version,
    channel: &ChannelName,
) -> Result<Version, Report<RepoVersionEvaluatorError>> {
    let next = tags
        .all
        .iter()
        .filter_map(|version| get_prerelease_number(version, target, channel))
        .max()
        .map_or(1, |highest| highest + 1);
    let text = format!("{channel}.{next}");
    let pre = Prerelease::new(&text)
        .change_context(RepoVersionEvaluatorError::Prerelease)
        .attach("prerelease", text)?;
    Ok(Version {
        pre,
        ..target.clone()
    })
}

/// Get N from a `<target>-<channel>.N` version.
///
/// - Returns `None` if `version` has a different core than `target` or another channel
fn get_prerelease_number(
    version: &Version,
    target: &Version,
    channel: &ChannelName,
) -> Option<u64> {
    if !version.has_same_core(target) {
        return None;
    }
    channel.get_number(version)
}

/// Errors returned by [`RepoVersionEvaluator`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum RepoVersionEvaluatorError {
    /// Unable to get the commits since the last release.
    #[error("Unable to get the commits since the last release")]
    Range,
    /// Unable to get the short hash for a development version.
    #[error("Unable to get the short hash for the development version")]
    ShortHash,
    /// A `Release-As` footer is not semver.
    #[error("Release-As footer is not a valid version")]
    ReleaseAsInvalid,
    /// A `Release-As` footer has a pre-release or build metadata.
    #[error("Release-As footer must not have a pre-release or build metadata")]
    ReleaseAsNotCore,
    /// A `Release-As` footer is not above the last full release.
    #[error("Release-As footer must be above the last full release")]
    ReleaseAsNotAbove,
    /// A `Release-As` footer names a version that is already tagged.
    #[error("Release-As footer names a version that is already tagged")]
    ReleaseAsTagged,
    /// The channel name does not form a valid pre-release.
    #[error("Unable to build the pre-release from the channel name")]
    Prerelease,
    /// The short hash does not form valid build metadata.
    #[error("Unable to build the build metadata from the short hash")]
    Build,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`RepoVersionEvaluator::evaluate`] bumps the base by the highest change on a release channel.
    #[test]
    fn repo_version_evaluator_evaluate_release() {
        // Arrange
        let git_commits = vec![
            GitCommit {
                hash: CommitHash::fixture("bbbbbbb2"),
                message: "fix: one".to_owned(),
                ..GitCommit::fixture()
            },
            GitCommit {
                message: "feat: two".to_owned(),
                ..GitCommit::fixture()
            },
        ];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), false)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.1.0"));
        assert_eq!(repo.stage, Stage::Release);
        assert_eq!(repo.channel, Some(ChannelName::fixture("release")));
        assert_eq!(repo.change, Change::Minor);
        assert!(!repo.tagged);
    }

    /// [`RepoVersionEvaluator::evaluate`] uses the full release tag on `HEAD` on a release channel.
    #[test]
    fn repo_version_evaluator_evaluate_release_tagged() {
        // Arrange
        let tags = VersionTags {
            all: vec![Version::fixture("1.0.0"), Version::fixture("1.1.0")],
            merged: vec![Version::fixture("1.0.0"), Version::fixture("1.1.0")],
            at_head: vec![Version::fixture("1.1.0")],
        };
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetVersionTagsStub { tags })
            .with_instance(GetCommitsStub {
                commits: Vec::new(),
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), false)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.1.0"));
        assert!(repo.tagged);
        assert_eq!(repo.change, Change::None);
    }

    /// [`RepoVersionEvaluator::evaluate`] appends `+dirty` on a release channel.
    #[test]
    fn repo_version_evaluator_evaluate_release_dirty() {
        // Arrange
        let git_commits = vec![GitCommit {
            message: "feat: one".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), true)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.1.0+dirty"));
    }

    /// [`RepoVersionEvaluator::evaluate`] numbers the pre-release one above the highest for the target.
    #[test]
    fn repo_version_evaluator_evaluate_prerelease() {
        // Arrange
        let tags = VersionTags {
            all: vec![
                Version::fixture("1.0.0"),
                Version::fixture("1.0.1-alpha.9"),
                Version::fixture("1.1.0-alpha.1"),
                Version::fixture("1.1.0-alpha.2"),
                Version::fixture("1.1.0-beta.7"),
            ],
            merged: vec![Version::fixture("1.0.0"), Version::fixture("1.1.0-alpha.1")],
            at_head: Vec::new(),
        };
        let git_commits = vec![GitCommit {
            message: "feat: one".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetVersionTagsStub { tags })
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture_prerelease();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), false)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.1.0-alpha.3"));
        assert_eq!(repo.stage, Stage::Prerelease);
        assert_eq!(repo.channel, Some(ChannelName::fixture("alpha")));
        assert!(!repo.tagged);
    }

    /// [`RepoVersionEvaluator::evaluate`] starts the pre-release number at 1.
    #[test]
    fn repo_version_evaluator_evaluate_prerelease_first() {
        // Arrange
        let git_commits = vec![GitCommit {
            message: "fix: one".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture_prerelease();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), false)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.0.1-alpha.1"));
    }

    /// [`RepoVersionEvaluator::evaluate`] reuses the newest matching pre-release tag on `HEAD`.
    #[test]
    fn repo_version_evaluator_evaluate_prerelease_tagged() {
        // Arrange
        let tags = VersionTags {
            all: vec![
                Version::fixture("1.0.0"),
                Version::fixture("1.1.0-alpha.1"),
                Version::fixture("1.1.0-alpha.2"),
            ],
            merged: vec![
                Version::fixture("1.0.0"),
                Version::fixture("1.1.0-alpha.1"),
                Version::fixture("1.1.0-alpha.2"),
            ],
            at_head: vec![
                Version::fixture("1.1.0-alpha.1"),
                Version::fixture("1.1.0-alpha.2"),
            ],
        };
        let git_commits = vec![GitCommit {
            message: "feat: one".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetVersionTagsStub { tags })
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture_prerelease();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), false)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.1.0-alpha.2"));
        assert!(repo.tagged);
    }

    /// [`RepoVersionEvaluator::evaluate`] appends `+dirty` to a pre-release.
    #[test]
    fn repo_version_evaluator_evaluate_prerelease_dirty() {
        // Arrange
        let git_commits = vec![GitCommit {
            message: "feat: one".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture_prerelease();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), true)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.1.0-alpha.1+dirty"));
    }

    /// [`RepoVersionEvaluator::evaluate`] builds a development version and bumps with no commits.
    #[test]
    fn repo_version_evaluator_evaluate_development() {
        // Arrange
        let tags = VersionTags {
            at_head: vec![Version::fixture("1.0.0")],
            ..VersionTags::fixture()
        };
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetVersionTagsStub { tags })
            .with_instance(GetCommitsStub {
                commits: Vec::new(),
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        // Act
        let repo = evaluator.evaluate(None, false).expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.0.1-DEV+abcdef1"));
        assert_eq!(repo.stage, Stage::Development);
        assert_eq!(repo.channel, None);
        assert_eq!(repo.change, Change::None);
        assert!(!repo.tagged);
    }

    /// [`RepoVersionEvaluator::evaluate`] appends `-dirty` to the development build metadata.
    #[test]
    fn repo_version_evaluator_evaluate_development_dirty() {
        // Arrange
        let git_commits = vec![GitCommit {
            message: "feat: one".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetVersionTagsStub {
                tags: VersionTags::default(),
            })
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        // Act
        let repo = evaluator.evaluate(None, true).expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("0.1.0-DEV+abcdef1-dirty"));
    }

    /// [`RepoVersionEvaluator::evaluate`] reports no change but still bumps a patch for chores.
    #[test]
    fn repo_version_evaluator_evaluate_no_change() {
        // Arrange
        let git_commits = vec![GitCommit {
            message: "chore: tidy".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), false)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.0.1"));
        assert_eq!(repo.change, Change::None);
    }

    /// [`RepoVersionEvaluator::evaluate`] with the target already released on another commit.
    #[test]
    fn repo_version_evaluator_evaluate_released_target() {
        // Arrange
        let tags = VersionTags {
            all: vec![
                Version::fixture("1.0.0"),
                Version::fixture("1.1.0"),
                Version::fixture("1.2.0-alpha.1"),
            ],
            ..VersionTags::fixture()
        };
        let git_commits = vec![GitCommit {
            message: "feat: one".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetVersionTagsStub { tags })
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), false)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("1.2.0"));
    }

    /// [`RepoVersionEvaluator::evaluate`] uses the newest `Release-As` footer, any key case.
    #[test]
    fn repo_version_evaluator_evaluate_release_as() {
        // Arrange
        let git_commits = vec![
            GitCommit {
                hash: CommitHash::fixture("bbbbbbb2"),
                message: "fix: newer\n\nrelease-as: 3.0.0".to_owned(),
                ..GitCommit::fixture()
            },
            GitCommit {
                message: "feat: older\n\nRelease-As: 2.0.0".to_owned(),
                ..GitCommit::fixture()
            },
        ];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let repo = evaluator
            .evaluate(Some(&channel), false)
            .expect("should get version");
        // Assert
        assert_eq!(repo.version, Version::fixture("3.0.0"));
        assert_eq!(repo.change, Change::Minor);
    }

    /// [`RepoVersionEvaluator::evaluate`] rejects a `Release-As` footer that is not semver.
    #[test]
    fn repo_version_evaluator_evaluate_release_as_invalid() {
        // Arrange
        let git_commits = vec![GitCommit {
            message: "feat: one\n\nRelease-As: two".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let report = evaluator
            .evaluate(Some(&channel), false)
            .expect_err("invalid Release-As should fail");
        // Assert
        assert_eq!(
            report.current_context(),
            &RepoVersionEvaluatorError::ReleaseAsInvalid
        );
    }

    /// [`RepoVersionEvaluator::evaluate`] rejects a `Release-As` footer with a pre-release or build metadata.
    #[test]
    fn repo_version_evaluator_evaluate_release_as_not_core() {
        for value in ["2.0.0-beta.1", "2.0.0+build"] {
            // Arrange
            let git_commits = vec![GitCommit {
                message: format!("feat: one\n\nRelease-As: {value}"),
                ..GitCommit::fixture()
            }];
            let evaluator = ServiceBuilder::new()
                .with_test_services()
                .with_instance(GetCommitsStub {
                    commits: git_commits,
                })
                .build()
                .expect::<RepoVersionEvaluator>();
            let channel = Channel::fixture();
            // Act
            let report = evaluator
                .evaluate(Some(&channel), false)
                .expect_err("Release-As with a pre-release or build should fail");
            // Assert
            assert_eq!(
                report.current_context(),
                &RepoVersionEvaluatorError::ReleaseAsNotCore,
                "{value}"
            );
        }
    }

    /// [`RepoVersionEvaluator::evaluate`] rejects a `Release-As` footer not above the base.
    #[test]
    fn repo_version_evaluator_evaluate_release_as_not_above() {
        // Arrange
        let git_commits = vec![GitCommit {
            message: "feat: one\n\nRelease-As: 1.0.0".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let report = evaluator
            .evaluate(Some(&channel), false)
            .expect_err("Release-As at the base should fail");
        // Assert
        assert_eq!(
            report.current_context(),
            &RepoVersionEvaluatorError::ReleaseAsNotAbove
        );
    }

    /// [`RepoVersionEvaluator::evaluate`] rejects a `Release-As` footer naming an existing tag.
    #[test]
    fn repo_version_evaluator_evaluate_release_as_tagged() {
        // Arrange
        let tags = VersionTags {
            all: vec![Version::fixture("1.0.0"), Version::fixture("2.0.0")],
            ..VersionTags::fixture()
        };
        let git_commits = vec![GitCommit {
            message: "feat: one\n\nRelease-As: 2.0.0".to_owned(),
            ..GitCommit::fixture()
        }];
        let evaluator = ServiceBuilder::new()
            .with_test_services()
            .with_instance(GetVersionTagsStub { tags })
            .with_instance(GetCommitsStub {
                commits: git_commits,
            })
            .build()
            .expect::<RepoVersionEvaluator>();
        let channel = Channel::fixture();
        // Act
        let report = evaluator
            .evaluate(Some(&channel), false)
            .expect_err("tagged Release-As should fail");
        // Assert
        assert_eq!(
            report.current_context(),
            &RepoVersionEvaluatorError::ReleaseAsTagged
        );
    }

    /// [`bump`] bumps a major change as a minor below `1.0.0`.
    #[test]
    fn bump_below_one() {
        // Arrange
        let cases = [
            (Version::new(1, 2, 3), Change::Major, Version::new(2, 0, 0)),
            (Version::new(1, 2, 3), Change::Minor, Version::new(1, 3, 0)),
            (Version::new(1, 2, 3), Change::Patch, Version::new(1, 2, 4)),
            (Version::new(1, 2, 3), Change::None, Version::new(1, 2, 4)),
            (Version::new(0, 2, 3), Change::Major, Version::new(0, 3, 0)),
            (Version::new(0, 2, 3), Change::Minor, Version::new(0, 3, 0)),
            (Version::new(0, 2, 3), Change::Patch, Version::new(0, 2, 4)),
            (Version::new(0, 0, 3), Change::Major, Version::new(0, 1, 0)),
            (Version::new(0, 0, 3), Change::Minor, Version::new(0, 1, 0)),
            (Version::new(0, 0, 3), Change::Patch, Version::new(0, 0, 4)),
        ];
        for (base, change, expected) in cases {
            // Act
            let target = bump(&base, change, &[]);
            // Assert
            assert_eq!(target, expected, "{base} {change}");
        }
    }
}
