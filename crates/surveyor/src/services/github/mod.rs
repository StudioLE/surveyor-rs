//! GitHub author lookup with the `gh` command.

mod author_resolver;
mod get_commit_author;
#[cfg(test)]
mod get_commit_author_spy;
mod get_repo_name;
#[cfg(test)]
mod get_repo_name_stub;
mod gh_adapter;

pub use author_resolver::*;
pub use get_commit_author::*;
#[cfg(test)]
pub use get_commit_author_spy::*;
pub use get_repo_name::*;
#[cfg(test)]
pub use get_repo_name_stub::*;
pub use gh_adapter::*;
