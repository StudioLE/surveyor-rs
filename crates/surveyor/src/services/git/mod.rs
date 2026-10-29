//! Adapter for the `git` command.

mod check_clean;
#[cfg(test)]
mod check_clean_stub;
mod get_commits;
#[cfg(test)]
mod get_commits_stub;
mod get_current_branch;
#[cfg(test)]
mod get_current_branch_stub;
mod get_last_commit_touching;
#[cfg(test)]
mod get_last_commit_touching_spy;
mod get_short_hash;
#[cfg(test)]
mod get_short_hash_stub;
mod get_tags_containing;
#[cfg(test)]
mod get_tags_containing_spy;
mod get_version_tags;
#[cfg(test)]
mod get_version_tags_stub;
mod git_adapter;
mod git_commit;
mod version_tags;

pub use check_clean::*;
#[cfg(test)]
pub use check_clean_stub::*;
pub use get_commits::*;
#[cfg(test)]
pub use get_commits_stub::*;
pub use get_current_branch::*;
#[cfg(test)]
pub use get_current_branch_stub::*;
pub use get_last_commit_touching::*;
#[cfg(test)]
pub use get_last_commit_touching_spy::*;
pub use get_short_hash::*;
#[cfg(test)]
pub use get_short_hash_stub::*;
pub use get_tags_containing::*;
#[cfg(test)]
pub use get_tags_containing_spy::*;
pub use get_version_tags::*;
#[cfg(test)]
pub use get_version_tags_stub::*;
pub use git_adapter::*;
pub use git_commit::*;
pub use version_tags::*;
