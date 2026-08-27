// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    println!("{:?}", args);
    #[cfg(debug_assertions)]
    if args.len() >= 3 && args.get(1).unwrap() == "--unwrap" {
        unsafe {
            std::env::set_var(
                "GTRS-UNWRAP-PATH",
                serde_json::to_string(&args[2..]).unwrap(),
            );
        }
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
