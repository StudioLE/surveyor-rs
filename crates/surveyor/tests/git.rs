//! Integration tests running [`GitAdapter`] against real git repos.

mod common;

use common::TestRepo;
use semver::Version;
use std::path::PathBuf;
use surveyor::prelude::*;

/// [`GetCurrentBranch::get_current_branch`] names the checked out branch.
#[test]
fn git_adapter_get_current_branch() {
    // Arrange
    let repo = TestRepo::new();
    repo.commit("chore: init");
    repo.git(&["switch", "--create", "release/1.x"]);
    let git = repo.get_services().expect::<GitAdapter>();
    // Act
    let branch = git.get_current_branch().expect("should get branch");
    // Assert
    assert_eq!(branch, "release/1.x");
}

/// [`GetCurrentBranch::get_current_branch`] fails on a detached `HEAD`.
#[test]
fn git_adapter_get_current_branch_detached() {
    // Arrange
    let repo = TestRepo::new();
    repo.commit("chore: init");
    repo.git(&["switch", "--detach"]);
    let git = repo.get_services().expect::<GitAdapter>();
    // Act
    let report = git
        .get_current_branch()
        .expect_err("detached HEAD should fail");
    // Assert
    assert_eq!(report.current_context(), &GetCurrentBranchError::Detached);
}

/// [`GetVersionTags::get_version_tags`] splits tags by reachability and ignores non-versions.
#[test]
fn git_adapter_get_version_tags() {
    // Arrange
    let repo = TestRepo::new();
    repo.commit("feat: one");
    repo.tag("v1.0.0");
    repo.tag("nightly");
    repo.tag("v1.0");
    repo.git(&["switch", "--create", "other"]);
    repo.commit("feat: other");
    repo.tag("v2.0.0");
    repo.git(&["switch", "main"]);
    repo.commit("feat: two");
    repo.tag("v1.1.0-alpha.1");
    let git = repo.get_services().expect::<GitAdapter>();
    // Act
    let tags = git.get_version_tags().expect("should get tags");
    // Assert
    let expected = VersionTags {
        all: vec![
            Version::fixture("1.0.0"),
            Version::fixture("1.1.0-alpha.1"),
            Version::fixture("2.0.0"),
        ],
        merged: vec![Version::fixture("1.0.0"), Version::fixture("1.1.0-alpha.1")],
        at_head: vec![Version::fixture("1.1.0-alpha.1")],
    };
    assert_eq!(tags, expected);
}

/// [`GetCommits::get_commits`] lists non-merge commits after the base, newest first.
#[test]
fn git_adapter_get_commits() {
    // Arrange
    let repo = TestRepo::new();
    repo.commit("feat: before");
    repo.tag("v1.0.0");
    repo.git(&["switch", "--create", "topic"]);
    let topic = repo.commit("fix: on topic\n\nBody line\n\nRefs: #1");
    repo.git(&["switch", "main"]);
    let main = repo.commit("feat: on main");
    repo.git(&["merge", "--no-ff", "--message", "Merge topic", "topic"]);
    let git = repo.get_services().expect::<GitAdapter>();
    // Act
    let commits = git
        .get_commits(Some(Version::fixture("1.0.0")))
        .expect("should get commits");
    // Assert
    let hashes: Vec<CommitHash> = commits.iter().map(|commit| commit.hash).collect();
    assert_eq!(hashes.len(), 2);
    assert!(hashes.contains(&topic));
    assert!(hashes.contains(&main));
    let topic = commits
        .iter()
        .find(|commit| commit.hash == topic)
        .expect("topic commit should be listed");
    assert_eq!(topic.message, "fix: on topic\n\nBody line\n\nRefs: #1");
    assert_eq!(topic.author_name, "Ada Lovelace");
    assert_eq!(topic.author_email, "ada@example.com");
}

/// [`GetCommits::get_commits`] lists every commit without a base.
#[test]
fn git_adapter_get_commits_without_base() {
    // Arrange
    let repo = TestRepo::new();
    let first = repo.commit("feat: first");
    let second = repo.commit("feat: second");
    let git = repo.get_services().expect::<GitAdapter>();
    // Act
    let commits = git.get_commits(None).expect("should get commits");
    // Assert
    let hashes: Vec<CommitHash> = commits.into_iter().map(|commit| commit.hash).collect();
    assert_eq!(hashes, [second, first]);
}

/// [`GetLastCommitTouching::get_last_commit_touching`] skips commits only touching excludes.
#[test]
fn git_adapter_get_last_commit_touching() {
    // Arrange
    let repo = TestRepo::new();
    repo.write("crate/src/lib.rs", "// one");
    let outer = repo.commit("feat: outer");
    repo.write("crate/nested/src/lib.rs", "// two");
    repo.commit("feat: nested");
    repo.write("other/src/lib.rs", "// three");
    repo.commit("feat: other");
    let git = repo.get_services().expect::<GitAdapter>();
    let paths = [repo.path().join("crate")];
    let excludes = [repo.path().join("crate/nested")];
    // Act
    let hash = git
        .get_last_commit_touching(&paths, &excludes)
        .expect("should get commit");
    // Assert
    assert_eq!(hash, Some(outer));
}

/// [`GetLastCommitTouching::get_last_commit_touching`] returns `None` for untouched paths.
#[test]
fn git_adapter_get_last_commit_touching_untouched() {
    // Arrange
    let repo = TestRepo::new();
    repo.write("crate/src/lib.rs", "// one");
    repo.commit("feat: outer");
    let git = repo.get_services().expect::<GitAdapter>();
    let paths = [PathBuf::from("missing")];
    // Act
    let hash = git
        .get_last_commit_touching(&paths, &[])
        .expect("should get commit");
    // Assert
    assert_eq!(hash, None);
}

/// [`GetTagsContaining::get_tags_containing`] lists tags whose history holds the commit.
#[test]
fn git_adapter_get_tags_containing() {
    // Arrange
    let repo = TestRepo::new();
    repo.commit("feat: one");
    repo.tag("v1.0.0");
    let second = repo.commit("feat: two");
    repo.tag("v1.1.0");
    repo.commit("feat: three");
    repo.tag("v1.2.0");
    let git = repo.get_services().expect::<GitAdapter>();
    // Act
    let tags = git.get_tags_containing(&second).expect("should get tags");
    // Assert
    assert_eq!(tags, [Version::fixture("1.1.0"), Version::fixture("1.2.0")]);
}

/// [`CheckClean::check_clean`] passes on a clean working tree.
#[test]
fn git_adapter_check_clean() {
    let repo = TestRepo::new();
    repo.commit("chore: init");
    let git = repo.get_services().expect::<GitAdapter>();
    git.check_clean().expect("clean tree should pass");
}

/// [`CheckClean::check_clean`] fails on an untracked file and names it.
#[test]
fn git_adapter_check_clean_untracked() {
    // Arrange
    let repo = TestRepo::new();
    repo.commit("chore: init");
    repo.write("untracked.txt", "new");
    let git = repo.get_services().expect::<GitAdapter>();
    // Act
    let report = git.check_clean().expect_err("untracked file should fail");
    // Assert
    assert_eq!(report.current_context(), &CheckCleanError::Dirty);
    let display = report.to_string();
    assert!(display.contains("untracked.txt"), "{display}");
}

/// [`GetShortHash::get_short_hash`] is the shortest prefix of `HEAD` in an unambiguous repo.
#[test]
fn git_adapter_get_short_hash() {
    // Arrange
    let repo = TestRepo::new();
    let hash = repo.commit("chore: init");
    let git = repo.get_services().expect::<GitAdapter>();
    // Act
    let short = git.get_short_hash().expect("should get hash");
    // Assert
    assert_eq!(short.as_str().len(), ShortCommitHash::MIN_LENGTH);
    assert!(hash.starts_with(&short), "{hash} {short}");
}
