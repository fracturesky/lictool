use clap::Parser;
use cli::Cli;
use util::errors::{display_error, LictoolResult};

/// A module to handle the command-line interface (CLI)
/// functionalities.
mod cli;

/// A module to store constants used throughout the application.
mod consts;

/// A module to manage SPDX-related operations and data.
mod spdx;

/// A module to handle template management.
mod template;

/// A module providing utility functions for various tasks.
mod util;

/// The entry point of the application.
#[tokio::main]
async fn main() -> LictoolResult<()> {
    let args = Cli::parse();
    if let Err(e) = args.exec_command().await {
        display_error(&e);
        std::process::exit(1);
    }
    Ok(())
}
