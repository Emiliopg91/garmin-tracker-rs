// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use garmin_tracker_rs_lib::check_running;
use std::process::exit;
use tauri_plugin_log::log::LevelFilter;

/// Binary entrypoint: handles the debug-only `--unwrap` dump mode, sets up env vars, and hands off to `run()`.
fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    #[cfg(debug_assertions)]
    if args.len() >= 3 && args.get(1).unwrap() == "--decode" {
        let paths = &args[2..];
        garmin_tracker_rs_lib::decode_files(paths);
        exit(0);
    } else {
        if std::env::var("IN_DEBUG").is_err() {
            tauri_rs_ts_ipc::build();
        }
    }

    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
    };

    if args.contains(&"--poll".to_string()) {
        exit(if check_running() { 1 } else { 0 });
    }

    let log_level = if args.contains(&"-v".to_string()) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    garmin_tracker_rs_lib::run(log_level);
}
