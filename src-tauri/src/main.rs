// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(debug_assertions)]
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command()]
struct Cli {
    /// Enable verbose mode to log extended information
    #[arg(short, long)]
    verbose: bool,

    /// Force usage of X11 instead of Wayland
    #[arg(short, long, default_value_t = true)]
    x11: bool,

    #[cfg(debug_assertions)]
    #[arg(short, long)]
    unwrap: bool,

    #[cfg(debug_assertions)]
    #[arg(long, requires = "unwrap")]
    path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    #[cfg(debug_assertions)]
    if cli.unwrap {
        match cli.path {
            Some(path) => unsafe {
                std::env::set_var("GTRS-UNWRAP-PATH", path.display().to_string());
            },
            None => {
                eprintln!("Error: --path is mandatory when using --unwrap");
                std::process::exit(1);
            }
        }
    } else {
        if std::env::var("IN_DEBUG").is_err() {
            tauri_rs_ts_ipc::build();
        }
    }

    unsafe {
        if cli.x11 {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        std::env::set_var("LOGGER_LEVEL", if cli.verbose { "Debug" } else { "" });
    }

    garmin_tracker_rs_lib::run();
}
