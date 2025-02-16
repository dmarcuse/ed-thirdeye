use std::path::PathBuf;

use clap::Parser;
use directories::ProjectDirs;
use log::info;

mod app;
mod panes;

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
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

impl Args {
    fn data_dir_or_default(&self) -> PathBuf {
        if let Some(p) = &self.data_dir {
            return p.to_owned();
        }

        let application = match cfg!(debug_assertions) {
            false => env!("CARGO_BIN_NAME"),
            true => concat!(env!("CARGO_BIN_NAME"), ".debug"),
        };

        // TODO: inform the user about this graphically as well
        let project_dir = ProjectDirs::from("", "", application)
            .expect("couldn't determine data directory - please set it manually");

        project_dir.data_local_dir().to_owned()
    }
}

fn main() -> eframe::Result {
    let args = Args::parse();
    env_logger::builder()
        .parse_filters(&args.log_filters)
        .init();
    info!("command-line arguments: {args:?}");
    app::start(args.data_dir_or_default())
}
