//! Lookup of the Cargo workspace and its members.

use crate::prelude::*;
use std::fs;
use std::process::Command;
use toml_edit::DocumentMut;

/// Get the Cargo workspace and its members.
pub trait GetWorkspace: Send + Sync {
    /// Get the workspace from `cargo metadata` and each member's `Cargo.toml`.
    fn get_workspace(&self) -> Result<Workspace, Report<GetWorkspaceError>>;
}

impl GetWorkspace for CargoAdapter {
    fn get_workspace(&self) -> Result<Workspace, Report<GetWorkspaceError>> {
        let mut command = Command::new("cargo");
        command
            .args([
                "metadata",
                "--no-deps",
                "--offline",
                "--format-version",
                "1",
            ])
            .current_dir(self.working_directory.path());
        let output = command.run_output(GetWorkspaceError::Spawn, GetWorkspaceError::Metadata)?;
        let metadata: Metadata = serde_json::from_slice(&output.stdout)
            .change_context(GetWorkspaceError::Parse)
            .attach("command", command.to_command_line())?;
        get_members(metadata)
    }
}

/// Build the [`Workspace`] from `cargo metadata` output.
///
/// - Counts only normal and build dependencies, not dev dependencies
fn get_members(metadata: Metadata) -> Result<Workspace, Report<GetWorkspaceError>> {
    let packages: Vec<MetadataPackage> = metadata
        .packages
        .into_iter()
        .filter(|package| metadata.workspace_members.contains(&package.id))
        .collect();
    let directories: BTreeMap<String, PathBuf> = packages
        .iter()
        .map(|package| {
            let directory = package
                .manifest_path
                .parent()
                .map_or_default(Path::to_path_buf);
            (package.name.clone(), directory)
        })
        .collect();
    let mut members = Vec::new();
    for package in packages {
        let internal = package.dependencies.iter().filter(|dependency| {
            let counted = matches!(dependency.kind.as_deref(), None | Some("build"));
            let is_member = dependency
                .path
                .as_ref()
                .is_some_and(|path| directories.get(&dependency.name) == Some(path));
            counted && is_member
        });
        let dependencies = internal
            .clone()
            .map(|dependency| dependency.name.clone())
            .collect();
        let unversioned_dependencies = internal
            .filter(|dependency| dependency.req == "*")
            .map(|dependency| dependency.name.clone())
            .collect();
        members.push(WorkspaceMember {
            inherits_version: get_inherits_version(&package.manifest_path)?,
            directory: directories.get(&package.name).cloned().unwrap_or_default(),
            publish: package
                .publish
                .is_none_or(|registries| !registries.is_empty()),
            name: package.name,
            manifest_path: package.manifest_path,
            dependencies,
            unversioned_dependencies,
        });
    }
    members.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Workspace {
        manifest_path: metadata.workspace_root.join("Cargo.toml"),
        members,
    })
}

/// Does the manifest at `path` set `version.workspace = true` in `[package]`?
fn get_inherits_version(path: &Path) -> Result<bool, Report<GetWorkspaceError>> {
    let text = fs::read_to_string(path)
        .change_context(GetWorkspaceError::Read)
        .attach_path(path)?;
    let document = DocumentMut::from_str(&text)
        .change_context(GetWorkspaceError::Manifest)
        .attach_path(path)?;
    let inherits = document
        .get("package")
        .and_then(|package| package.get("version"))
        .and_then(|version| version.get("workspace"))
        .and_then(toml_edit::Item::as_bool)
        .unwrap_or(false);
    Ok(inherits)
}

/// Errors returned by [`GetWorkspace::get_workspace`].
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GetWorkspaceError {
    /// Unable to start `cargo`.
    #[error("Unable to start cargo")]
    Spawn,
    /// `cargo metadata` exited with an error.
    #[error("Unable to read the workspace with cargo metadata")]
    Metadata,
    /// `cargo metadata` output is not the expected JSON.
    #[error("Unable to parse cargo metadata output")]
    Parse,
    /// Unable to read a member manifest.
    #[error("Unable to read crate manifest")]
    Read,
    /// A member manifest is not valid TOML.
    #[error("Unable to parse crate manifest")]
    Manifest,
}
