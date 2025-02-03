use clap::Parser;
use log::{debug, info};

mod journal;

/// Command-line arguments for the program
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Log message filter string - see https://docs.rs/env_logger for syntax
    #[arg(long = "log", env = "RUST_LOG")]
    #[cfg_attr(debug_assertions, arg(default_value = "debug"))]
    #[cfg_attr(not(debug_assertions), arg(default_value = concat!(env!("CARGO_PKG_NAME"), "=info,warn")))]
    log_filters: String,
}

fn main() {
    let args = Args::parse();
    env_logger::builder()
        .parse_filters(&args.log_filters)
        .init();

    debug!("Command-line arguments: {args:#?}");

    let path = journal::get_default_journal_path();
    info!("Default journal path: {path:?}");
}
