mod dao;
mod dto;
mod logic;
mod mtp;
mod parser;
mod utils;

#[cfg(debug_assertions)]
use std::path::Path;
use std::{process::exit, sync::RwLock, time::Duration};

use rusqlite_orm::database::builder::{DatabaseConnectionBuilder, JournalMode};
use rusqlite_orm_macros::dlls;
use tauri::Manager;
use tauri_plugin_log::{
    Target, TargetKind,
    log::{LevelFilter, debug, error, info},
};

use crate::{
    dto::app::Settings,
    logic::{
        app::{
            export_database, get_environment, get_settings, get_translations,
            notify_frontend_ready, update_settings_value,
        },
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

#[cfg(debug_assertions)]
pub fn unwrap_path<P>(paths: &[P])
where
    P: AsRef<Path>,
{
    for path in paths {
        let entries = read_from_file(path).unwrap();
        debug_dump(path, &entries);
    }
}

/// Boots the Tauri app: acquires the single-instance lock, opens/migrates the DB, loads settings, and registers commands.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    SingleInstance::acquire();

    if let Err(e) = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
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
        .setup(move |app| {
            info!(
                "Starting {} v{} with PID {}",
                *constants::APP_NAME,
                *constants::APP_VERSION,
                *constants::PID
            );

            debug!("Initializing database...");
            let builder = DatabaseConnectionBuilder::default()
                .location(constants::DB_FILE.clone())
                .busy_timeout(Duration::from_secs(8))
                .connection_timeout(Duration::from_secs(5))
                .pool_size(10)
                .min_idle(10)
                .enable_foreign_keys()
                .journal_mode(JournalMode::Delete);
            match builder.build("gtrs") {
                Ok(database) => {
                    if let Err(e) = database.create_schema(&DDLS) {
                        error!("Could not initialize database: {}", e);
                        exit(constants::ExitCodes::DbError.into())
                    }
                    debug!("Loading settings...");
                    let settings = Settings {
                        auto_sync: crate::dao::settings::Settings::get_auto_sync(&database),
                        distance_unit: crate::dao::settings::Settings::get_distance_unit(&database),
                        language: crate::dao::settings::Settings::get_language(&database),
                        start_boot: crate::dao::settings::Settings::get_start_on_boot(&database),
                        weight_unit: crate::dao::settings::Settings::get_weight_unit(&database),
                    };

                    app.manage(database);
                    app.manage(RwLock::new(settings));
                }
                Err(e) => {
                    error!("Could not open database: {}", e);
                    exit(constants::ExitCodes::DbError.into())
                }
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
            get_body_measures,
            add_body_measures,
            get_environment,
            delete_body_metric,
            get_settings,
            update_settings_value,
            export_database,
            get_translations
        ])
        .run(tauri::generate_context!())
    {
        eprintln!("Error while running tauri application {}", e);
        exit(constants::ExitCodes::TauriError.into())
    }
}
