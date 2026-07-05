// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Cli {
    /// Enable verbose mode to log extended information
    #[arg(short, long)]
    verbose: bool,

    /// Force usage of Wayland
    #[arg(short, long, default_value_t = true)]
    x11: bool,
}

fn main() {
    #[cfg(debug_assertions)]
    if std::env::var("IN_DEBUG").is_err() {
        tauri_rs_ts_ipc::build();
    }
    let cli = Cli::parse();

    unsafe {
        if cli.x11 {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        std::env::set_var("LOGGER_LEVEL", if cli.verbose { "Debug" } else { "" });
    }

    garmin_tracker_rs_lib::run();
}
