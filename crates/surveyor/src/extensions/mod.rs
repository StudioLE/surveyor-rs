//! Extension traits on existing types.

mod bytes_to_trimmed_string;
mod command_run_output;
mod command_to_command_line;
mod item_set_string;
mod output_ok_or_report;
mod path_normalize;
mod report_attach_response;
mod str_is_hex;
mod str_normalize_type;
#[cfg(any(test, feature = "testing"))]
mod version_fixture;
mod version_has_same_core;
mod version_is_release;

pub use bytes_to_trimmed_string::*;
pub use command_run_output::*;
pub use command_to_command_line::*;
pub use item_set_string::*;
pub use output_ok_or_report::*;
pub use path_normalize::*;
pub use report_attach_response::*;
pub use str_is_hex::*;
pub use str_normalize_type::*;
#[cfg(any(test, feature = "testing"))]
pub use version_fixture::*;
pub use version_has_same_core::*;
pub use version_is_release::*;
