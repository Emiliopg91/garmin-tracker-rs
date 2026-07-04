// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Cli {
    /// Enable verbose mode to log extended information
    #[arg(short, long)]
    verbose: bool,

    /// Force usage of X11
    #[arg(short, long)]
    x11: bool,
}

fn main() {
    let cli = Cli::parse();

    unsafe {
        if cli.x11 {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        std::env::set_var("LOGGER_LEVEL", if cli.verbose { "Debug" } else { "" });
    }

    #[cfg(debug_assertions)]
    if std::env::var("IN_DEBUG").is_err() {
        tauri_rs_ts_ipc::build();
    }

    garmin_tracker_rs_lib::run();
}
