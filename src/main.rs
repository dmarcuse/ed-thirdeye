use std::path::PathBuf;

use clap::Parser;
use log::info;

mod app;
mod panes;

/// Get the default data directory, for when none is specified
fn get_default_data_dir() -> PathBuf {
    let suffix = match cfg!(debug_assertions) {
        true => concat!(env!("CARGO_BIN_NAME"), ".debug"),
        false => env!("CARGO_BIN_NAME"),
    };
    dirs::data_local_dir()
        .expect("user's local data directory must be known")
        .join(suffix)
}

/// Command-line arguments for the program
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Log message filter string - see <https://docs.rs/env_logger> for syntax
    #[arg(long = "log", env = "RUST_LOG")]
    #[cfg_attr(debug_assertions, arg(default_value = "debug"))]
    #[cfg_attr(not(debug_assertions), arg(default_value = concat!(env!("CARGO_PKG_NAME"), "=info,warn")))]
    log_filters: String,

    /// Directory under which to store application state - will be created if
    /// necessary
    #[arg(long, default_value_os_t = get_default_data_dir())]
    data_dir: PathBuf,
}

fn main() -> eframe::Result {
    let args = Args::parse();
    env_logger::builder()
        .parse_filters(&args.log_filters)
        .init();
    info!("command-line arguments: {args:?}");
    app::start(args.data_dir)
}
