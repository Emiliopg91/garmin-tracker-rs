// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{Datelike, Local, Timelike};

fn main() {
    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    let time = Local::now();
    println!(
        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}]",
        time.year(),
        time.month(),
        time.day(),
        time.hour(),
        time.minute(),
        time.second(),
        time.timestamp_subsec_millis()
    );

    #[cfg(debug_assertions)]
    tauri_rs_ts_ipc::build();

    garmin_tracker_rs_lib::run()
}
