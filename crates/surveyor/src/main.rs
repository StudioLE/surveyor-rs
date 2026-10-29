//! Binary entrypoint for `surveyor`.

use std::process::ExitCode;
use surveyor::prelude::*;

fn main() -> ExitCode {
    Cli::new().run()
}
