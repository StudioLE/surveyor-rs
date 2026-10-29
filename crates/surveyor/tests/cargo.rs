//! Integration tests running [`CargoAdapter`] against real Cargo workspaces.

mod common;

use common::TestRepo;
use insta::assert_snapshot;
use semver::Version;
use std::collections::BTreeSet;
use std::fs;
use std::process::Command;
use surveyor::prelude::*;

/// Root manifest with an internal workspace dependency and a comment.
const ROOT: &str = r#"[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
# Set by CI
version = "0.0.0"
edition = "2024"

[workspace.dependencies]
alpha = { path = "crates/alpha", version = "0.0.0" } # internal
"#;

/// Crate inheriting the workspace version.
const ALPHA: &str = r#"[package]
name = "alpha"
version.workspace = true
edition.workspace = true
"#;

/// Crate with its own version, a workspace dependency, and a path-only dev dependency.
const BETA: &str = r#"[package]
name = "beta"
version = "0.0.0" # own
edition.workspace = true

[dependencies]
alpha = { workspace = true }

[dev-dependencies]
gamma = { path = "../gamma" }
"#;

/// Unpublished crate with a path-only dependency.
const GAMMA: &str = r#"[package]
name = "gamma"
version = { workspace = true }
edition.workspace = true
publish = false

[build-dependencies]
beta = { path = "../beta" }
"#;

/// [`GetWorkspace::get_workspace`] reads members, inheritance, publish, and internal dependencies.
#[test]
fn cargo_adapter_get_workspace() {
    // Arrange
    let repo = fixture_workspace();
    let cargo = repo.get_services().expect::<CargoAdapter>();
    // Act
    let workspace = cargo.get_workspace().expect("should get workspace");
    // Assert
    let root = repo.path().canonicalize().expect("should canonicalize");
    assert_eq!(workspace.manifest_path, root.join("Cargo.toml"));
    let expected = vec![
        WorkspaceMember {
            name: "alpha".to_owned(),
            manifest_path: root.join("crates/alpha/Cargo.toml"),
            directory: root.join("crates/alpha"),
            inherits_version: true,
            publish: true,
            dependencies: BTreeSet::new(),
            unversioned_dependencies: BTreeSet::new(),
        },
        WorkspaceMember {
            name: "beta".to_owned(),
            manifest_path: root.join("crates/beta/Cargo.toml"),
            directory: root.join("crates/beta"),
            inherits_version: false,
            publish: true,
            dependencies: BTreeSet::from(["alpha".to_owned()]),
            unversioned_dependencies: BTreeSet::new(),
        },
        WorkspaceMember {
            name: "gamma".to_owned(),
            manifest_path: root.join("crates/gamma/Cargo.toml"),
            directory: root.join("crates/gamma"),
            inherits_version: true,
            publish: false,
            dependencies: BTreeSet::from(["beta".to_owned()]),
            unversioned_dependencies: BTreeSet::from(["beta".to_owned()]),
        },
    ];
    assert_eq!(workspace.members, expected);
}

/// [`ManifestVersionWriter::write`] edits manifests read by `cargo metadata`, keeping comments.
#[test]
fn manifest_version_writer_write() {
    // Arrange
    let repo = fixture_workspace();
    let services = repo.get_services();
    let cargo = services.expect::<CargoAdapter>();
    let writer = services.expect::<ManifestVersionWriter>();
    let workspace = cargo.get_workspace().expect("should get workspace");
    let repo_version = Version::fixture("1.2.0-alpha.3");
    let crates = vec![
        CrateVersion {
            version: repo_version.clone(),
            ..CrateVersion::fixture()
        },
        CrateVersion {
            name: "beta".to_owned(),
            version: Version::fixture("1.1.0"),
            bumped: false,
            ..CrateVersion::fixture()
        },
        CrateVersion {
            name: "gamma".to_owned(),
            version: repo_version.clone(),
            ..CrateVersion::fixture()
        },
    ];
    // Act
    writer
        .write(&workspace, &crates, &repo_version, false)
        .expect("should write versions");
    // Assert
    assert_snapshot!("root", read(&repo, "Cargo.toml"));
    assert_snapshot!("beta", read(&repo, "crates/beta/Cargo.toml"));
    assert_eq!(read(&repo, "crates/alpha/Cargo.toml"), ALPHA);
    assert_eq!(read(&repo, "crates/gamma/Cargo.toml"), GAMMA);
    cargo
        .get_workspace()
        .expect("written manifests should still load");
}

/// [`LockVersionWriter::write`] keeps `Cargo.lock` passing `--locked` for every version stage.
#[test]
fn lock_version_writer_write() {
    for text in ["1.2.0", "1.2.0-alpha.3", "1.2.0-DEV+abcdef1"] {
        // Arrange
        let repo = fixture_workspace();
        cargo(&repo, &["generate-lockfile", "--offline"]);
        let services = repo.get_services();
        let workspace = services
            .expect::<CargoAdapter>()
            .get_workspace()
            .expect("should get workspace");
        let manifests = services.expect::<ManifestVersionWriter>();
        let lock = services.expect::<LockVersionWriter>();
        let repo_version = Version::fixture(text);
        let crates = vec![
            CrateVersion {
                version: repo_version.clone(),
                ..CrateVersion::fixture()
            },
            CrateVersion {
                name: "beta".to_owned(),
                version: Version::fixture("1.1.0"),
                bumped: false,
                ..CrateVersion::fixture()
            },
            CrateVersion {
                name: "gamma".to_owned(),
                version: repo_version.clone(),
                ..CrateVersion::fixture()
            },
        ];
        manifests
            .write(&workspace, &crates, &repo_version, false)
            .expect("should write manifests");
        // Act
        lock.write(&workspace, &crates, false)
            .expect("should write lock file");
        // Assert
        cargo(
            &repo,
            &["metadata", "--locked", "--offline", "--format-version", "1"],
        );
    }
}

/// [`CrateVersionEvaluator::evaluate`] gives the new version to a changed crate and every crate depending on it.
#[test]
fn crate_version_evaluator_evaluate_dependency_chain() {
    // Arrange
    let repo = fixture_chain_workspace();
    repo.tag("v1.0.0");
    repo.write("crates/core/src/lib.rs", "pub fn core() {}\n");
    repo.commit("fix: core");
    let services = repo.get_services();
    let workspace = services
        .expect::<CargoAdapter>()
        .get_workspace()
        .expect("should get workspace");
    let evaluator = services.expect::<CrateVersionEvaluator>();
    let repo_version = RepoVersion {
        version: Version::fixture("1.0.1-DEV+abcdef1"),
        stage: Stage::Development,
        channel: None,
        change: Change::Patch,
        tagged: false,
    };
    // Act
    let versions = evaluator
        .evaluate(&workspace, &repo_version)
        .expect("should get versions");
    // Assert
    let expected = vec![
        CrateVersion {
            name: "app".to_owned(),
            version: repo_version.version.clone(),
            ..CrateVersion::fixture()
        },
        CrateVersion {
            name: "cli".to_owned(),
            version: repo_version.version.clone(),
            ..CrateVersion::fixture()
        },
        CrateVersion {
            name: "core".to_owned(),
            version: repo_version.version.clone(),
            ..CrateVersion::fixture()
        },
        CrateVersion {
            name: "macros".to_owned(),
            version: Version::fixture("1.0.0"),
            bumped: false,
            ..CrateVersion::fixture()
        },
    ];
    assert_eq!(versions, expected);
}

/// Write a three crate workspace into a new repo.
fn fixture_workspace() -> TestRepo {
    let repo = TestRepo::new();
    repo.write("Cargo.toml", ROOT);
    for (name, manifest) in [("alpha", ALPHA), ("beta", BETA), ("gamma", GAMMA)] {
        repo.write(&format!("crates/{name}/Cargo.toml"), manifest);
        repo.write(&format!("crates/{name}/src/lib.rs"), "");
    }
    repo.commit("chore: init");
    repo.tag("v1.0.0");
    repo
}

/// Write a workspace where `cli` depends on `app`, `app` on `core`, and `macros` on nothing.
fn fixture_chain_workspace() -> TestRepo {
    let repo = TestRepo::new();
    repo.write(
        "Cargo.toml",
        "[workspace]\nresolver = \"3\"\nmembers = [\"crates/*\"]\n",
    );
    for (name, dependency) in [
        ("app", Some("core")),
        ("cli", Some("app")),
        ("core", None),
        ("macros", None),
    ] {
        let dependencies = dependency.map_or_default(|dependency| {
            format!("\n[dependencies]\n{dependency} = {{ path = \"../{dependency}\" }}\n")
        });
        let manifest = format!(
            "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n{dependencies}"
        );
        repo.write(&format!("crates/{name}/Cargo.toml"), &manifest);
        repo.write(&format!("crates/{name}/src/lib.rs"), "");
    }
    repo.commit("chore: init");
    repo
}

/// Run `cargo` in the repo, failing the test if it exits unsuccessfully.
fn cargo(repo: &TestRepo, args: &[&str]) {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(repo.path())
        .output()
        .expect("should run cargo");
    assert!(
        output.status.success(),
        "cargo {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Read `path` relative to the repo.
fn read(repo: &TestRepo, path: &str) -> String {
    fs::read_to_string(repo.path().join(path)).expect("should read")
}
