mod constants;
mod dao;
mod dto;
mod logic;
mod mtp;
mod parser;

use std::process::exit;

use rusqlite_orm::database::DATABASE_INST;
use rusqlite_orm_macros::dlls;
use tauri::Manager;
use tauri_plugin_log::{
    Target, TargetKind,
    log::{LevelFilter, debug, error, info},
};

use crate::logic::{
    app::{get_environment, notify_frontend_ready},
    exercises::{get_exercise_details, get_exercises},
    sessions::{get_session_details, get_sessions, import_from_device, save_session_changes},
    user::{add_user_measures, get_user_measures},
    workouts::{get_workout_details, get_workout_list},
};

#[cfg(debug_assertions)]
use crate::parser::{debug_dump, read_from_file};

dlls!("../resources/ddl");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    if let Ok(path) = std::env::var("GTRS-UNWRAP-PATH") {
        let entries = read_from_file(&path).unwrap();
        debug_dump(&path, &entries);
        exit(0);
    }

    let res = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Warn)
                .level_for(constants::LIB_NAME.clone(), *constants::LOG_LEVEL)
                .level_for("command", *constants::LOG_LEVEL)
                .level_for("rusqlite_orm", *constants::LOG_LEVEL)
                .target(Target::new(TargetKind::Folder {
                    path: constants::LOGS_DIR.clone(),
                    file_name: None,
                }))
                .max_file_size(constants::LOG_FILE_MAX_SIZE)
                .rotation_strategy(constants::LOG_FILE_ROTATION_STRATEGY)
                .format(|out, message, record| {
                    let mut target = record.target();
                    target = if target.len() > 30 {
                        &target[target.len() - 30..]
                    } else {
                        target
                    };

                    out.finish(format_args!(
                        "[{}][{:<30}][{:<5.5}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
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
        .plugin(tauri_plugin_opener::init())
        .setup(move |_| {
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
            if let Err(e) = db.create_schema(&DDLS) {
                error!("Could not initialize database: {}", e);
                exit(constants::ExitCodes::DbError.into())
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
            get_workout_list,
            get_workout_details,
            import_from_device,
            notify_frontend_ready,
            get_user_measures,
            add_user_measures,
            get_environment,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = res {
        eprintln!("Error while running tauri application {}", e);
        exit(constants::ExitCodes::TauriError.into())
    }
}
