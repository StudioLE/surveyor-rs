//! Writing of crate versions into `Cargo.lock`.

use crate::prelude::*;
use toml_edit::{DocumentMut, Item};

/// Write crate versions into `Cargo.lock`.
#[derive(FromServices)]
pub struct LockVersionWriter {
    read_lock: Arc<dyn ReadLock>,
    write_lock: Arc<dyn WriteLock>,
}

impl LockVersionWriter {
    /// Set the version of each workspace package in `Cargo.lock`, preserving formatting.
    ///
    /// - Skips if the workspace has no `Cargo.lock`
    /// - Treats entries without a `source` as workspace packages
    /// - Keeps build metadata, as Cargo does
    /// - Logs each edit and skips the write if `dry_run`
    pub fn write(
        &self,
        workspace: &Workspace,
        crates: &[CrateVersion],
        dry_run: bool,
    ) -> Result<(), Report<LockVersionWriterError>> {
        let path = workspace.manifest_path.with_file_name("Cargo.lock");
        let text = self
            .read_lock
            .read_lock(&path)
            .change_context(LockVersionWriterError::Read)?;
        let Some(text) = text else {
            debug!(path = %path.display(), "Skipping lock file as it doesn't exist");
            return Ok(());
        };
        let mut document = DocumentMut::from_str(&text)
            .change_context(LockVersionWriterError::Parse)
            .attach_path(&path)?;
        edit(&mut document, &path, crates);
        if dry_run {
            info!(path = %path.display(), "Skipping lock file write for dry run");
            return Ok(());
        }
        self.write_lock
            .write_lock(&path, &document.to_string())
            .change_context(LockVersionWriterError::Write)
    }
}

/// Set the `version` of each `[[package]]` entry without a `source` to its crate's version.
fn edit(document: &mut DocumentMut, path: &Path, crates: &[CrateVersion]) {
    let Some(packages) = document
        .get_mut("package")
        .and_then(Item::as_array_of_tables_mut)
    else {
        return;
    };
    for package in packages.iter_mut() {
        if package.contains_key("source") {
            continue;
        }
        let Some(name) = package.get("name").and_then(Item::as_str) else {
            continue;
        };
        let Some(crate_version) = crates.iter().find(|version| version.name == name) else {
            continue;
        };
        let Some(item) = package.get_mut("version") else {
            continue;
        };
        let version = crate_version.version.to_string();
        info!(path = %path.display(), package = crate_version.name, version, "Setting version");
        item.set_string(&version);
    }
}

/// Errors returned by [`LockVersionWriter`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum LockVersionWriterError {
    /// Unable to read `Cargo.lock`.
    #[error("Unable to read lock file")]
    Read,
    /// `Cargo.lock` is not valid TOML.
    #[error("Unable to parse lock file")]
    Parse,
    /// Unable to write `Cargo.lock`.
    #[error("Unable to write lock file")]
    Write,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`LockVersionWriter::write`] sets the versions of workspace packages, keeping build metadata.
    ///
    /// - Leaves the registry `beta` untouched as it has a `source`
    #[test]
    fn lock_version_writer_write() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(ReadLockStub {
                lock: Some(LOCK.to_owned()),
            })
            .build();
        let writer = services.expect::<LockVersionWriter>();
        let crates = vec![
            CrateVersion {
                version: Version::fixture("1.2.0-DEV+abcdef1"),
                ..CrateVersion::fixture()
            },
            CrateVersion {
                name: "beta".to_owned(),
                version: Version::fixture("1.1.0"),
                bumped: false,
                ..CrateVersion::fixture()
            },
        ];
        // Act
        writer
            .write(&Workspace::fixture(), &crates, false)
            .expect("should write versions");
        // Assert
        let locks = services.expect::<WriteLockSpy>().get_locks();
        let expected =
            BTreeMap::from([(PathBuf::from("/repo/Cargo.lock"), LOCK_WRITTEN.to_owned())]);
        assert_eq!(locks, expected);
    }

    /// [`LockVersionWriter::write`] writes nothing on a dry run.
    #[test]
    fn lock_version_writer_write_dry_run() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(ReadLockStub {
                lock: Some(LOCK.to_owned()),
            })
            .build();
        let writer = services.expect::<LockVersionWriter>();
        let crates = vec![CrateVersion::fixture()];
        // Act
        writer
            .write(&Workspace::fixture(), &crates, true)
            .expect("dry run should succeed");
        // Assert
        let locks = services.expect::<WriteLockSpy>().get_locks();
        assert_eq!(locks, BTreeMap::new());
    }

    /// [`LockVersionWriter::write`] writes nothing when the workspace has no `Cargo.lock`.
    #[test]
    fn lock_version_writer_write_missing() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(ReadLockStub { lock: None })
            .build();
        let writer = services.expect::<LockVersionWriter>();
        let crates = vec![CrateVersion::fixture()];
        // Act
        writer
            .write(&Workspace::fixture(), &crates, false)
            .expect("missing lock file should succeed");
        // Assert
        let locks = services.expect::<WriteLockSpy>().get_locks();
        assert_eq!(locks, BTreeMap::new());
    }

    /// [`LockVersionWriter::write`] fails when `Cargo.lock` isn't valid TOML.
    #[test]
    fn lock_version_writer_write_invalid() {
        // Arrange
        let services = ServiceBuilder::new()
            .with_test_services()
            .with_instance(ReadLockStub {
                lock: Some("[[package]\n".to_owned()),
            })
            .build();
        let writer = services.expect::<LockVersionWriter>();
        let crates = vec![CrateVersion::fixture()];
        // Act
        let report = writer
            .write(&Workspace::fixture(), &crates, false)
            .expect_err("invalid lock file should fail");
        // Assert
        assert_eq!(report.current_context(), &LockVersionWriterError::Parse);
        let locks = services.expect::<WriteLockSpy>().get_locks();
        assert_eq!(locks, BTreeMap::new());
    }

    const LOCK: &str = r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "alpha"
version = "0.0.0"

[[package]]
name = "beta"
version = "0.0.0"

[[package]]
name = "beta"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    const LOCK_WRITTEN: &str = r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "alpha"
version = "1.2.0-DEV+abcdef1"

[[package]]
name = "beta"
version = "1.1.0"

[[package]]
name = "beta"
version = "0.9.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
}
