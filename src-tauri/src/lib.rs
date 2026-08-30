mod dao;
mod dto;
mod logic;
mod mtp;
mod parser;
mod utils;

use std::{process::exit, sync::RwLock};

use rusqlite_orm::{
    dao::{Repository, helpers::types::where_clause::Where},
    database::Database,
};
use rusqlite_orm_macros::dlls;
use tauri_plugin_log::{
    Target, TargetKind,
    log::{LevelFilter, debug, error, info},
};

use crate::{
    dao::{
        coordinates::{self, CoordinatesRepository},
        session::{self, SessionRepository},
    },
    dto::{
        app::Settings,
        notifications::{NotificationDefinition, NotificationKind},
    },
    logic::{
        app::{
            SETTINGS_INST, export_database, get_environment, get_settings, get_translations,
            notify_frontend_ready, update_settings_value,
        },
        body_metrics::{add_body_measures, delete_body_metric, get_body_measures},
        exercises::{get_exercise_details, get_exercises},
        notifications::show_notification,
        sessions::{
            get_location_from_coordinates, get_session_details, get_sessions, import_from_device,
            save_session_changes,
        },
        workouts::{get_workout_details, get_workout_list},
    },
    utils::{constants, single_instance::SingleInstance, translations::translate},
};

#[cfg(debug_assertions)]
use crate::parser::{debug_dump, read_from_file};

dlls!("../resources/ddl");

/// Boots the Tauri app: acquires the single-instance lock, opens/migrates the DB, loads settings, and registers commands.
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
            if let Err(e) = Database::run_in_transaction(|tx| {
                let sessions = SessionRepository::select()
                    .where_(Where::Eq(session::entity::columns::WORKOUT, "".into()))
                    .fetch_in(tx)?;

                if !sessions.is_empty() {
                    let gps_coordinates = CoordinatesRepository::select()
                        .where_(Where::In(
                            coordinates::entity::columns::SESSION,
                            sessions.iter().map(|s| s.date.into()).collect::<Vec<_>>(),
                        ))
                        .fetch_in(tx)?;

                    if !gps_coordinates.is_empty() {
                        show_notification(NotificationDefinition {
                            title: translate("aligning_database"),
                            body: translate("operation_may_last"),
                            kind: NotificationKind::Temporal,
                        });
                        let lang = SETTINGS_INST
                            .get()
                            .unwrap()
                            .read()
                            .unwrap()
                            .language
                            .to_string();

                        for gps_coords in gps_coordinates {
                            let coords: Vec<(i32, i32)> = (&gps_coords).into();
                            if let Some(start_point) = coords.first() {
                                let location = get_location_from_coordinates(
                                    start_point.0 as f64 * constants::SEMICIRCLE_TO_DEGREES,
                                    start_point.1 as f64 * constants::SEMICIRCLE_TO_DEGREES,
                                    &lang,
                                );
                                SessionRepository::update()
                                    .set(session::entity::columns::WORKOUT, location.into())
                                    .where_(Where::Eq(
                                        session::entity::columns::DATE,
                                        gps_coords.session.into(),
                                    ))
                                    .execute_in(tx)?;
                            }
                        }
                    }
                }

                Ok(())
            }) {
                error!("Error aligning database: {}", e);
                exit(constants::ExitCodes::DbError.into())
            }

            debug!("Loading settings...");
            SETTINGS_INST
                .set(RwLock::new(Settings {
                    auto_sync: crate::dao::settings::Settings::get_auto_sync(),
                    distance_unit: crate::dao::settings::Settings::get_distance_unit(),
                    language: crate::dao::settings::Settings::get_language(),
                    start_boot: crate::dao::settings::Settings::get_start_on_boot(),
                    weight_unit: crate::dao::settings::Settings::get_weight_unit(),
                }))
                .unwrap();

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
        .run(tauri::generate_context!());

    if let Err(e) = res {
        eprintln!("Error while running tauri application {}", e);
        exit(constants::ExitCodes::TauriError.into())
    }
}
