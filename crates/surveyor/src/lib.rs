//! Version a Cargo repo from git tags and conventional commits, and render release notes.

mod app;
mod commands;
mod extensions;
pub mod prelude;
mod schema;
mod services;
#[cfg(test)]
mod testing_prelude;
