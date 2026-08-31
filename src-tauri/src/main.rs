// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Binary entrypoint: handles the debug-only `--unwrap` dump mode, sets up env vars, and hands off to `run()`.
fn main() {
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
    }

    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
        std::env::set_var(
            "LOGGER_LEVEL",
            if args.contains(&"-v".to_string()) {
                "Debug"
            } else {
                ""
            },
        );
    }

    garmin_tracker_rs_lib::run();
}
