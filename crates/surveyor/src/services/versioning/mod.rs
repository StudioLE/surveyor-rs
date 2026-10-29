//! Version calculation from tags and commits.

mod commit_range;
mod commit_range_provider;
mod repo_version;
mod repo_version_evaluator;

pub use commit_range::*;
pub use commit_range_provider::*;
pub use repo_version::*;
pub use repo_version_evaluator::*;
