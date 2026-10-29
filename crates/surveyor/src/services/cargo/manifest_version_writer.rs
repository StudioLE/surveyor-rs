//! Writing of crate versions into Cargo manifests.

use crate::prelude::*;
use semver::BuildMetadata;
use std::iter;
use toml_edit::{DocumentMut, Item, TableLike};

/// Dependency tables of a member manifest whose internal entries get versions.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "build-dependencies", "dev-dependencies"];

/// Write crate versions into Cargo manifests.
#[derive(FromServices)]
pub struct ManifestVersionWriter {
    read_manifest: Arc<dyn ReadManifest>,
    write_manifest: Arc<dyn WriteManifest>,
}

impl ManifestVersionWriter {
    /// Set the versions in the root and member manifests, preserving formatting and comments.
    ///
    /// - Sets `[workspace.package] version` to `repo_version` if any member inherits it
    /// - Sets each member's own `version`
    /// - Sets `version` on internal dependency entries that have both `path` and `version`
    /// - Computes every edit before writing any file
    /// - Writes only the manifests that changed
    /// - Logs each edit and skips every write if `dry_run`
    pub fn write(
        &self,
        workspace: &Workspace,
        crates: &[CrateVersion],
        repo_version: &Version,
        dry_run: bool,
    ) -> Result<(), Report<ManifestVersionWriterError>> {
        let mut manifests = self.read_manifests(workspace)?;
        let versions = get_dependency_versions(workspace, crates);
        edit_root_manifest(&mut manifests, workspace, repo_version, &versions);
        edit_member_manifests(&mut manifests, workspace, crates, &versions);
        self.write_changed_manifests(&manifests, dry_run)
    }

    /// Read and parse the root and member manifests by path.
    ///
    /// - Reads each path once, as the root manifest is also a member when it has a `[package]`
    fn read_manifests(
        &self,
        workspace: &Workspace,
    ) -> Result<BTreeMap<PathBuf, Manifest>, Report<ManifestVersionWriterError>> {
        let mut manifests = BTreeMap::new();
        let paths = iter::once(&workspace.manifest_path)
            .chain(workspace.members.iter().map(|member| &member.manifest_path));
        for path in paths {
            if manifests.contains_key(path) {
                continue;
            }
            let original = self
                .read_manifest
                .read_manifest(path)
                .change_context(ManifestVersionWriterError::Read)?;
            let document = DocumentMut::from_str(&original)
                .change_context(ManifestVersionWriterError::Parse)
                .attach_path(path)?;
            manifests.insert(path.clone(), Manifest { original, document });
        }
        Ok(manifests)
    }

    /// Write each manifest whose text differs from the original.
    ///
    /// - Logs each changed manifest and skips the write if `dry_run`
    fn write_changed_manifests(
        &self,
        manifests: &BTreeMap<PathBuf, Manifest>,
        dry_run: bool,
    ) -> Result<(), Report<ManifestVersionWriterError>> {
        for (path, manifest) in manifests {
            let text = manifest.document.to_string();
            if text == manifest.original {
                continue;
            }
            if dry_run {
                info!(path = %path.display(), "Skipping manifest write for dry run");
                continue;
            }
            self.write_manifest
                .write_manifest(path, &text)
                .change_context(ManifestVersionWriterError::Write)?;
        }
        Ok(())
    }
}

/// Parsed manifest alongside the text it was parsed from.
struct Manifest {
    original: String,
    document: DocumentMut,
}

/// Edit the root manifest, setting the workspace version only if a member inherits it.
fn edit_root_manifest(
    manifests: &mut BTreeMap<PathBuf, Manifest>,
    workspace: &Workspace,
    repo_version: &Version,
    versions: &BTreeMap<PathBuf, Version>,
) {
    let Some(manifest) = manifests.get_mut(&workspace.manifest_path) else {
        return;
    };
    let inherits = workspace
        .members
        .iter()
        .any(|member| member.inherits_version);
    edit_root(
        &mut manifest.document,
        &workspace.manifest_path,
        inherits.then_some(repo_version),
        versions,
    );
}

/// Edit each member manifest, setting its own version only if it doesn't inherit one.
fn edit_member_manifests(
    manifests: &mut BTreeMap<PathBuf, Manifest>,
    workspace: &Workspace,
    crates: &[CrateVersion],
    versions: &BTreeMap<PathBuf, Version>,
) {
    for member in &workspace.members {
        let Some(manifest) = manifests.get_mut(&member.manifest_path) else {
            continue;
        };
        let own_version = crates
            .iter()
            .find(|version| version.name == member.name && !member.inherits_version)
            .map(|version| &version.version);
        edit_member(
            &mut manifest.document,
            &member.manifest_path,
            own_version,
            versions,
        );
    }
}

/// Map each member directory to its version without build metadata.
fn get_dependency_versions(
    workspace: &Workspace,
    crates: &[CrateVersion],
) -> BTreeMap<PathBuf, Version> {
    workspace
        .members
        .iter()
        .filter_map(|member| {
            let crate_version = crates.iter().find(|version| version.name == member.name)?;
            let mut version = crate_version.version.clone();
            version.build = BuildMetadata::EMPTY;
            Some((member.directory.clone(), version))
        })
        .collect()
}

/// Set `[workspace.package] version` and the `[workspace.dependencies]` versions.
fn edit_root(
    document: &mut DocumentMut,
    path: &Path,
    workspace_version: Option<&Version>,
    versions: &BTreeMap<PathBuf, Version>,
) {
    let Some(workspace) = document.get_mut("workspace") else {
        return;
    };
    if let Some(version) = workspace_version
        && let Some(package) = workspace
            .get_mut("package")
            .and_then(Item::as_table_like_mut)
    {
        set_version(package, path, "workspace.package.version", version);
    }
    if let Some(dependencies) = workspace
        .get_mut("dependencies")
        .and_then(Item::as_table_like_mut)
    {
        set_dependencies(dependencies, path, "workspace.dependencies", versions);
    }
}

/// Set the member's own `version` and its internal dependency versions.
///
/// - Includes the dependency tables of each `[target.'cfg(…)']` table
fn edit_member(
    document: &mut DocumentMut,
    path: &Path,
    own_version: Option<&Version>,
    versions: &BTreeMap<PathBuf, Version>,
) {
    if let Some(version) = own_version
        && let Some(package) = document
            .get_mut("package")
            .and_then(Item::as_table_like_mut)
    {
        set_version(package, path, "package.version", version);
    }
    set_dependency_tables(document.as_table_mut(), path, "", versions);
    let Some(targets) = document.get_mut("target").and_then(Item::as_table_like_mut) else {
        return;
    };
    for (platform, target) in targets.iter_mut() {
        if let Some(target) = target.as_table_like_mut() {
            let prefix = format!("target.{}.", platform.get());
            set_dependency_tables(target, path, &prefix, versions);
        }
    }
}

/// Set the internal dependency versions in each of the [`DEPENDENCY_TABLES`] in `parent`.
fn set_dependency_tables(
    parent: &mut dyn TableLike,
    path: &Path,
    prefix: &str,
    versions: &BTreeMap<PathBuf, Version>,
) {
    for name in DEPENDENCY_TABLES {
        if let Some(dependencies) = parent.get_mut(name).and_then(Item::as_table_like_mut) {
            set_dependencies(dependencies, path, &format!("{prefix}{name}"), versions);
        }
    }
}

/// Set `version` on each entry with both `path` and `version` pointing at a member.
///
/// - Leaves path-only entries untouched
fn set_dependencies(
    dependencies: &mut dyn TableLike,
    path: &Path,
    table: &str,
    versions: &BTreeMap<PathBuf, Version>,
) {
    let directory = path.parent().unwrap_or(Path::new(""));
    for (name, entry) in dependencies.iter_mut() {
        let Some(entry) = entry.as_table_like_mut() else {
            continue;
        };
        let Some(relative) = entry.get("path").and_then(Item::as_str) else {
            continue;
        };
        if !entry.contains_key("version") {
            continue;
        }
        let Some(version) = versions.get(&directory.join(relative).normalize()) else {
            continue;
        };
        set_version(entry, path, &format!("{table}.{}", name.get()), version);
    }
}

/// Set `version` in `table`, logging the edit.
fn set_version(table: &mut dyn TableLike, path: &Path, key: &str, version: &Version) {
    let text = version.to_string();
    info!(path = %path.display(), key, version = text, "Setting version");
    match table.get_mut("version") {
        Some(item) => item.set_string(&text),
        None => {
            table.insert("version", toml_edit::value(text));
        }
    }
}

/// Errors returned by [`ManifestVersionWriter`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ManifestVersionWriterError {
    /// Unable to read a manifest.
    #[error("Unable to read manifest")]
    Read,
    /// A manifest is not valid TOML.
    #[error("Unable to parse manifest")]
    Parse,
    /// Unable to write a manifest.
    #[error("Unable to write manifest")]
    Write,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_prelude::*;

    /// [`ManifestVersionWriter::write`] sets every version, keeps comments, and writes only changed manifests.
    #[test]
    fn manifest_version_writer_write() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(ReadManifestStub {
                manifests: get_manifests(),
            })
            .build();
        let writer = services.expect::<ManifestVersionWriter>();
        let crates = vec![
            CrateVersion {
                name: "inherit".to_owned(),
                version: Version::fixture("1.2.0-DEV+abcdef1"),
                ..CrateVersion::fixture()
            },
            CrateVersion {
                name: "own".to_owned(),
                version: Version::fixture("1.1.0"),
                bumped: false,
                ..CrateVersion::fixture()
            },
            CrateVersion {
                name: "path-only".to_owned(),
                version: Version::fixture("1.2.0-DEV+abcdef1"),
                ..CrateVersion::fixture()
            },
        ];
        let version = Version::fixture("1.2.0-DEV+abcdef1");
        // Act
        writer
            .write(&Workspace::fixture_manifests(), &crates, &version, false)
            .expect("should write versions");
        // Assert
        let written = services.expect::<WriteManifestSpy>().get_manifests();
        let paths: Vec<&Path> = written.keys().map(PathBuf::as_path).collect();
        assert_eq!(
            paths,
            [
                Path::new("/repo/Cargo.toml"),
                Path::new("/repo/crates/own/Cargo.toml")
            ]
        );
        let root = written.get(Path::new("/repo/Cargo.toml"));
        let own = written.get(Path::new("/repo/crates/own/Cargo.toml"));
        assert_snapshot!("root", root.expect("root should be written"));
        assert_snapshot!("own", own.expect("own should be written"));
    }

    /// [`ManifestVersionWriter::write`] writes nothing on a dry run.
    #[test]
    fn manifest_version_writer_write_dry_run() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(ReadManifestStub {
                manifests: get_manifests(),
            })
            .build();
        let writer = services.expect::<ManifestVersionWriter>();
        let crates = vec![CrateVersion {
            name: "own".to_owned(),
            version: Version::fixture("1.1.0"),
            ..CrateVersion::fixture()
        }];
        let version = Version::fixture("1.2.0");
        // Act
        writer
            .write(&Workspace::fixture_manifests(), &crates, &version, true)
            .expect("dry run should succeed");
        // Assert
        let written = services.expect::<WriteManifestSpy>().get_manifests();
        assert_eq!(written, BTreeMap::new());
    }

    /// [`ManifestVersionWriter::write`] fails when a manifest can't be read.
    #[test]
    fn manifest_version_writer_write_missing() {
        // Arrange
        let writer = ServiceBuilder::new()
            .with_test_services()
            .with_instance(ReadManifestStub {
                manifests: BTreeMap::new(),
            })
            .build()
            .expect::<ManifestVersionWriter>();
        let version = Version::fixture("1.2.0");
        // Act
        let report = writer
            .write(&Workspace::fixture_manifests(), &[], &version, false)
            .expect_err("missing manifest should fail");
        // Assert
        assert_eq!(report.current_context(), &ManifestVersionWriterError::Read);
    }

    const ROOT: &str = r#"[workspace]
members = ["crates/*"]

[workspace.package]
# Set by CI
version = "0.0.0"

[workspace.dependencies]
inherit = { path = "crates/inherit", version = "0.0.0" } # internal
own = { version = "0.0.0", path = "crates/own" }
path-only = { path = "crates/path-only" }
serde = { version = "1.0.229" }
"#;

    const OWN: &str = r#"[package]
name = "own"
version = "0.0.0" # kept

[dependencies]
inherit = { workspace = true }

[dev-dependencies.path-only]
path = "../path-only"
version = "0.0.0"

[target.'cfg(unix)'.dependencies]
inherit = { path = "../inherit", version = "0.0.0" }
"#;

    const INHERIT: &str = "[package]\nname = \"inherit\"\nversion.workspace = true\n";

    /// Manifest text of [`Workspace::fixture_manifests`] by path.
    fn get_manifests() -> BTreeMap<PathBuf, String> {
        [
            ("/repo/Cargo.toml", ROOT),
            ("/repo/crates/inherit/Cargo.toml", INHERIT),
            ("/repo/crates/own/Cargo.toml", OWN),
            ("/repo/crates/path-only/Cargo.toml", INHERIT),
        ]
        .into_iter()
        .map(|(path, text)| (PathBuf::from(path), text.to_owned()))
        .collect()
    }
}
