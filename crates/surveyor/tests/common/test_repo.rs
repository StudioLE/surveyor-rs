//! Temporary git repo for integration tests.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use studiole_di::prelude::*;
use surveyor::prelude::*;
use tempfile::TempDir;

/// Temporary git repo on `main`, isolated from the user's git config.
pub struct TestRepo {
    directory: TempDir,
}

impl TestRepo {
    /// Create an empty repo on `main`.
    pub fn new() -> Self {
        let directory = TempDir::new().expect("should create temp dir");
        let repo = Self { directory };
        repo.git(&["init", "--initial-branch", "main"]);
        repo
    }

    /// Path of the repo.
    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    /// Write `content` to `path` relative to the repo, creating parent directories.
    pub fn write(&self, path: &str, content: &str) {
        let path = self.path().join(path);
        let parent = path.parent().expect("path should have a parent");
        fs::create_dir_all(parent).expect("should create parent dirs");
        fs::write(&path, content).expect("should write file");
    }

    /// Stage every change and commit with `message`, returning the full hash.
    pub fn commit(&self, message: &str) -> CommitHash {
        self.git(&["add", "--all"]);
        self.git(&["commit", "--allow-empty", "--message", message]);
        CommitHash::from_str(self.git(&["rev-parse", "HEAD"]).trim()).expect("HEAD should parse")
    }

    /// Create a lightweight tag on `HEAD`.
    pub fn tag(&self, name: &str) {
        self.git(&["tag", name]);
    }

    /// Run `git` in the repo with a fixed identity and no user config, returning stdout.
    pub fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(self.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "Ada Lovelace")
            .env("GIT_AUTHOR_EMAIL", "ada@example.com")
            .env("GIT_COMMITTER_NAME", "Ada Lovelace")
            .env("GIT_COMMITTER_EMAIL", "ada@example.com")
            .output()
            .expect("should run git");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("git output should be UTF-8")
    }

    /// Service provider with the adapters pointed at the repo.
    pub fn get_services(&self) -> ServiceProvider {
        ServiceBuilder::new()
            .with_instance(WorkingDirectory::new(self.path().to_path_buf()))
            .with_type::<CargoAdapter>()
            .with_type::<GitAdapter>()
            .with_trait::<dyn GetLastCommitTouching, GitAdapter>()
            .with_trait::<dyn GetTagsContaining, GitAdapter>()
            .with_trait::<dyn ReadLock, CargoAdapter>()
            .with_trait::<dyn ReadManifest, CargoAdapter>()
            .with_trait::<dyn WriteLock, CargoAdapter>()
            .with_trait::<dyn WriteManifest, CargoAdapter>()
            .with_type::<CrateVersionEvaluator>()
            .with_type::<LockVersionWriter>()
            .with_type::<ManifestVersionWriter>()
            .build()
    }
}
