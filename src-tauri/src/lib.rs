mod dao;
mod dto;
mod logic;
mod mtp;
mod parser;
mod utils;

use std::process::exit;

use rusqlite_orm::database::Database;
use rusqlite_orm_macros::dlls;
use tauri_plugin_log::{
    Target, TargetKind,
    log::{LevelFilter, debug, error, info},
};
use tokio::sync::RwLock;

use crate::{
    dto::app::Settings,
    logic::{
        app::{SETTINGS_INST, get_environment, get_settings, notify_frontend_ready},
        body_metrics::{add_body_measures, delete_body_metric, get_body_measures},
        exercises::{get_exercise_details, get_exercises},
        sessions::{get_session_details, get_sessions, import_from_device, save_session_changes},
        workouts::{get_workout_details, get_workout_list},
    },
    utils::{constants, single_instance::SingleInstance},
};

#[cfg(debug_assertions)]
use crate::parser::{debug_dump, read_from_file};

dlls!("../resources/ddl");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    SingleInstance::acquire();

    #[cfg(debug_assertions)]
    if let Ok(paths) = std::env::var("GTRS-UNWRAP-PATH") {
        let paths = serde_json::from_str::<Vec<String>>(&paths).unwrap();
        for path in paths {
            let entries = read_from_file(&path).unwrap();
            debug_dump(&path, &entries);
        }
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
        .plugin(tauri_plugin_opener::init())
        .setup(move |_| {
            info!(
                "Starting {} v{} with PID {}",
                *constants::APP_NAME,
                *constants::APP_VERSION,
                *constants::PID
            );

            debug!("Initializing database...");
            if let Err(e) = Database::initialize(constants::DB_FILE.clone()) {
                error!("Could not open database: {}", e);
                exit(constants::ExitCodes::DbError.into())
            };
            if let Err(e) = Database::create_schema(&DDLS) {
                error!("Could not initialize database: {}", e);
                exit(constants::ExitCodes::DbError.into())
            }

            debug!("Loading settings...");
            SETTINGS_INST.set(RwLock::new(Settings::load())).unwrap();

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
            get_body_measures,
            add_body_measures,
            get_environment,
            delete_body_metric,
            get_settings
        ])
        .run(tauri::generate_context!());

    if let Err(e) = res {
        eprintln!("Error while running tauri application {}", e);
        exit(constants::ExitCodes::TauriError.into())
    }
}
