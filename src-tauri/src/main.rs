// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let mut level = "";
    if std::env::args()
        .collect::<Vec<String>>()
        .contains(&"--verbose".to_string())
    {
        level = "Debug"
    }

    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
        std::env::set_var("LOGGER_LEVEL", level);
    }

    #[cfg(debug_assertions)]
    tauri_rs_ts_ipc::build();

    garmin_tracker_rs_lib::run();
}
