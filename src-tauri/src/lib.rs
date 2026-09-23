mod dao;
mod dto;
mod logic;
mod mtp;
mod parser;
mod rclone;
mod udev;
mod utils;

#[cfg(debug_assertions)]
use std::path::Path;
use std::{fs, process::exit, sync::RwLock, time::Duration};

use rusqlite_orm::database::{
    DatabasePool,
    builder::{DatabaseConnectionBuilder, JournalMode},
};
use rusqlite_orm::ddls;
use tauri::{Manager, WindowEvent};
use tauri_plugin_log::{
    Target, TargetKind,
    log::{LevelFilter, debug, error, info, warn},
};

use crate::{
    dto::app::Settings, logic::{
        app::{
            export_database, get_environment, get_settings, get_translations,
            notify_frontend_ready, update_settings_value, upload_to_cloud,
        }, body_metrics::{add_body_measures, delete_body_metric, get_body_measures}, exercises::{get_exercise_details, get_exercises}, sessions::{
            _import_from_files, export_gpx, get_session_details, get_sessions, import_from_device, import_from_files, recalculate_e1rm, recalculate_prs, save_session_changes,
        }, workouts::{get_workout_details, get_workout_list, set_workout_status},
    }, udev::UdevManager, utils::{constants, single_instance::SingleInstance},
};

#[cfg(debug_assertions)]
use crate::parser::FitParser;

#[cfg(debug_assertions)]
pub fn decode_files<P>(paths: &[P])
where
    P: AsRef<Path>,
{
    for path in paths {
        let res: Result<(), Box<dyn std::error::Error>> = match FitParser::from_file(path) {
            Ok(stream) => match stream.debug_dump() {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Err(e) => Err(Box::new(e)),
        };
        if let Err(e) = res {
            eprintln!(
                "Error handling '{}': \n  {}",
                path.as_ref().display(),
                e.as_ref()
            )
        }
    }
}

pub fn check_running() -> bool {
    SingleInstance::is_app_running().0
}

pub fn write_mtp_rules(auto_run: bool) -> crate::udev::errors::Result<()> {
    if !fs::exists(constants::RULE_FILE).unwrap() {
        info!("Installing udev rules...");
        UdevManager::write_rules_file(auto_run)?;

        Ok(())
    } else {
        Ok(())
    }
}

pub fn force_write_mtp_rules(auto_run: bool) -> crate::udev::errors::Result<()> {
    UdevManager::write_rules_file(auto_run)
}

pub type SettingsLock = RwLock<Settings>;

/// Boots the Tauri app: acquires the single-instance lock, opens/migrates the DB, loads settings, and registers commands.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(log_level: LevelFilter) {
    SingleInstance::acquire();

    if let Err(e) = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Warn)
                .level_for(constants::LIB_NAME.clone(), log_level)
                .level_for("command", log_level)
                .level_for("rusqlite_orm", log_level)
                .clear_targets()
                .target(Target::new(TargetKind::Folder {
                    path: constants::LOGS_DIR.clone(),
                    file_name: None,
                }))
                .target(
                    Target::new(TargetKind::Stdout).format(|out, message, _record| {
                        let message = message.to_string();
                        let char_count = message.chars().count();
                        let message = if char_count > 1000 {
                            let truncated: String = message.chars().take(1000).collect();
                            format!("{}... and {} more", truncated, char_count - 1000)
                        } else {
                            message
                        };

                        out.finish(format_args!("{}", message))
                    }),
                )
                .max_file_size(constants::LOG_FILE_MAX_SIZE)
                .rotation_strategy(constants::LOG_FILE_ROTATION_STRATEGY.clone())
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

            if let Err(e) = write_mtp_rules(false) {
                eprintln!("Error installing udev rules {e}");
                exit(constants::ExitCodes::UdevError.into())
            }

            fn initialize() -> (DatabasePool, Settings) {
                debug!("Initializing database...");
                let already_exists = fs::exists(constants::DB_FILE.clone()).unwrap();
                let builder = DatabaseConnectionBuilder::default()
                    .location(constants::DB_FILE.clone())
                    .busy_timeout(Duration::from_secs(8))
                    .connection_timeout(Duration::from_secs(5))
                    .pool_size(2)
                    .min_idle(2)
                    .enable_foreign_keys()
                    .journal_mode(JournalMode::Delete);
                match builder.build("gtrs") {
                    Ok(database) => {
                        let mut ddls = ddls!("../resources/ddl");
                        ddls[4].update_fn=Some(recalculate_e1rm);
                        ddls[5].update_fn=Some(recalculate_prs);

                        if let Err(e) = database.create_schema(&ddls) {
                            error!("Could not initialize database: {}", e);
                            exit(constants::ExitCodes::DbError.into())
                        }

                        let version = crate::dao::settings::Settings::get_version(&database);
                        if already_exists && version.major == 0 {
                            warn!("Detected incompatible database schema version, cleaning up...");
                            drop(database);
                            let _ = fs::remove_file(constants::DB_FILE.clone());
                            initialize()
                        } else {
                            debug!("Loading settings...");
                            let settings = Settings::from(&database);

                            crate::dao::settings::Settings::set_version(
                                &database,
                                &constants::APP_SEM_VERSION.clone(),
                            )
                            .unwrap();

                            (database, settings)
                        }
                    }
                    Err(e) => {
                        error!("Could not open database: {}", e);
                        exit(constants::ExitCodes::DbError.into())
                    }
                }
            }

            let (database, settings) = initialize();

            app.manage(database);
            app.manage(SettingsLock::new(settings));

            let app_handle = app.handle().clone();
            let window = app_handle.get_webview_window("main");
            if let Some(window) = window {
                window.on_window_event(move |event| {
                    if let WindowEvent::DragDrop(event) = event
                        && let tauri::DragDropEvent::Drop { paths, position: _ } = event
                    {
                        let paths = paths
                            .iter()
                            .filter(|p| p.display().to_string().to_lowercase().ends_with(".fit"))
                            .cloned()
                            .collect::<Vec<_>>();

                        info!("Dropped {} .fit files: {:?}", paths.len(), paths);

                        let _ = _import_from_files(app_handle.clone(), paths.as_slice());
                    }
                });
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
            get_translations,
            upload_to_cloud,
            import_from_files,
            set_workout_status,
            export_gpx
        ])
        .run(tauri::generate_context!())
    {
        eprintln!("Error while running tauri application {}", e);
        exit(constants::ExitCodes::TauriError.into())
    }
}
