// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_log::log::LevelFilter;

/// Binary entrypoint: handles the debug-only `--unwrap` dump mode, sets up env vars, and hands off to `run()`.
fn main() {
    let mut log_level = LevelFilter::Info;

    let args = std::env::args().collect::<Vec<String>>();

    #[cfg(debug_assertions)]
    if args.len() >= 3 && args.get(1).unwrap() == "--unwrap" {
        use std::process::exit;

        let paths = &args[2..];
        garmin_tracker_rs_lib::unwrap_path(paths);
        exit(0);
    } else {
        if std::env::var("IN_DEBUG").is_err() {
            tauri_rs_ts_ipc::build();
        }

        if args.contains(&"-v".to_string()) {
            log_level = LevelFilter::Debug
        }
    }

    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
    };

    garmin_tracker_rs_lib::run(log_level);
}
