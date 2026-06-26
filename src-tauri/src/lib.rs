use std::process::exit;

use chrono::{Datelike, Local, Timelike};
use tauri::Manager;
use tauri_plugin_log::{
    Target, TargetKind,
    log::{LevelFilter, debug, error, info},
};

use crate::{
    garmin::database::DATABASE_INST,
    ui::{
        devices::start_device_watcher,
        exercises::{get_exercise_details, get_exercises},
        sessions::{
            get_session_details, get_sessions, import_from_device, import_from_file,
            save_session_changes,
        },
        workouts::{get_workout_details, get_workout_list},
    },
};

mod constants;
mod garmin;
mod ui;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let res = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(*constants::LOG_LEVEL)
                .level_for("nusb", LevelFilter::Warn)
                .target(Target::new(TargetKind::Folder {
                    path: constants::LOGS_DIR.clone(),
                    file_name: None,
                }))
                .max_file_size(constants::LOG_FILE_MAX_SIZE)
                .rotation_strategy(constants::LOG_FILE_ROTATION_STRATEGY)
                .format(|out, message, record| {
                    let time = Local::now();
                    let mut target = record.target();
                    target = if target.len() > 30 {
                        &target[target.len() - 30..]
                    } else {
                        target
                    };

                    out.finish(format_args!(
                        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}][{:<30}][{:<5.5}] {}",
                        time.year(),
                        time.month(),
                        time.day(),
                        time.hour(),
                        time.minute(),
                        time.second(),
                        time.timestamp_subsec_millis(),
                        target,
                        record.level().to_string(),
                        message
                    ))
                })
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let window = app.get_webview_window("main").expect("no main window");
            let _ = window.unminimize();
            let _ = window.set_focus();
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            info!(
                "Starting {} v{}",
                *constants::APP_NAME,
                *constants::APP_VERSION
            );

            debug!("Initializing database...");
            let mut db = DATABASE_INST.lock().unwrap();
            if let Err(e) = db.open(constants::DB_FILE.clone()) {
                error!("Could not open database: {}", e);
                exit(constants::ExitCodes::DbError.into())
            }
            if let Err(e) = db.create_schema() {
                error!("Could not initialize database: {}", e);
                exit(constants::ExitCodes::DbError.into())
            }

            debug!("Showing up main window...");
            if let Some(window) = app.get_webview_window("main") {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    let _ = window.show();
                });
            } else {
                error!("Could not find main window instance");
                exit(constants::ExitCodes::NoMainWindow.into())
            }

            debug!("Setup finished");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_session_details,
            save_session_changes,
            get_exercises,
            get_exercise_details,
            import_from_file,
            get_workout_list,
            get_workout_details,
            import_from_device,
            start_device_watcher
        ])
        .run(tauri::generate_context!());

    if let Err(e) = res {
        eprintln!("Error while running tauri application {}", e);
        exit(constants::ExitCodes::TauriError.into())
    }
}
