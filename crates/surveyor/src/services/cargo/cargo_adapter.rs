//! Adapter wrapping the `cargo` CLI and Cargo manifests.

use crate::prelude::*;

/// Adapter wrapping the `cargo` CLI and Cargo manifests in the [`WorkingDirectory`].
#[derive(FromServices)]
pub struct CargoAdapter {
    /// Directory `cargo` runs in.
    pub(super) working_directory: Arc<WorkingDirectory>,
}
