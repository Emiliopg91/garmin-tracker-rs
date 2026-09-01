use std::{collections::HashMap, fs, sync::RwLock};

use chrono::Local;
use garmin_tracker_rs_macros::traced_command;
use rusqlite_orm::database::DatabaseConnection;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    constants,
    dto::{
        app::{AppEnvironment, Settings},
        export::Export,
        notifications::{NotificationDefinition, NotificationKind},
    },
    logic::{
        devices::start_device_watcher, notifications::show_notification,
        sessions::update_pending_geolocation,
    },
    utils::translations::{Languages, TRANSLATIONS, translate, translate_and_replace},
};
use tauri_plugin_log::log::{error, info};

/// Returns the current in-memory app settings.
#[traced_command]
#[tauri::command]
pub async fn get_settings(settings: State<'_, RwLock<Settings>>) -> Result<Settings, String> {
    Ok(settings.read().unwrap().clone())
}

/// Called once the frontend has mounted: starts the USB device watcher and reveals the main window.
#[traced_command]
#[tauri::command]
pub async fn notify_frontend_ready(app: AppHandle, webview_window: WebviewWindow) {
    info!("UI ready");

    start_device_watcher(app.clone());

    info!("Showing up main window...");
    let _ = webview_window.set_title(&format!(
        "{} v{}",
        webview_window.title().unwrap(),
        *constants::APP_VERSION
    ));
    let _ = webview_window.show();

    let app = app.clone();
    std::thread::spawn(move || {
        let db = app.state::<DatabaseConnection>();
        update_pending_geolocation(&app, &db);
    });
}

/// Reports whether this is a debug or release build.
#[traced_command]
#[tauri::command]
pub async fn get_environment() -> AppEnvironment {
    if cfg!(debug_assertions) {
        AppEnvironment::Debug
    } else {
        AppEnvironment::Release
    }
}

/// Updates a single named setting: persists it to the DB, applies any side effect (e.g. autostart toggle), and refreshes `SETTINGS_INST`.
#[traced_command]
#[tauri::command]
pub async fn update_settings_value(
    app: AppHandle,
    database: State<'_, DatabaseConnection>,
    settings: State<'_, RwLock<Settings>>,
    name: &str,
    value: &str,
) -> Result<(), String> {
    match name {
        crate::dao::settings::settings_keys::AUTO_SYNC => {
            let value = value == "true";
            crate::dao::settings::Settings::set_auto_sync(&database, value).map_err(|e| e.to_string())?;
            settings.write().unwrap().auto_sync = value;
        }
        crate::dao::settings::settings_keys::DISTANCE_UNIT => {
            let value = value.try_into()?;
            crate::dao::settings::Settings::set_distance_unit(&database, &value)
                .map_err(|e| e.to_string())?;
            settings.write().unwrap().distance_unit = value;
        }
        crate::dao::settings::settings_keys::START_ON_BOOT => {
            let value = value == "true";
            crate::dao::settings::Settings::set_start_on_boot(&database, value)
                .map_err(|e| e.to_string())?;
            if value {
                app.autolaunch().enable()
            } else {
                app.autolaunch().disable()
            }
            .map_err(|e| e.to_string())?;
            settings.write().unwrap().start_boot = value;
        }
        crate::dao::settings::settings_keys::WEIGHT_UNIT => {
            let value = value.try_into()?;
            crate::dao::settings::Settings::set_weight_unit(&database, &value)
                .map_err(|e| e.to_string())?;
            settings.write().unwrap().weight_unit = value;
        }
        crate::dao::settings::settings_keys::LANGUAGE => {
            let value = Languages::from_name(value);
            crate::dao::settings::Settings::set_language(&database, &value).map_err(|e| e.to_string())?;
            settings.write().unwrap().language = value;
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Exports the whole database to a timestamped JSON file in the user's home directory and notifies on success/failure.
#[traced_command]
#[tauri::command]
pub fn export_database(
    database: State<'_, DatabaseConnection>,
    settings: State<'_, RwLock<Settings>>,
) -> Result<(), String> {
    let path = constants::HOME_DIR.join(format!(
        "{}-{}.json",
        *constants::APP_NAME,
        Local::now().format("%Y-%m-%d-%H-%M-%S")
    ));
    info!("Exporting database to {}...", path.display());

    let res = Export::from_database(&database)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        .and_then(|export| {
            export
                .to_json()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
        .and_then(|json| {
            fs::write(&path, json).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        });

    let lang = settings.read().unwrap().language;
    match res {
        Ok(()) => {
            show_notification(NotificationDefinition {
                title: translate("ok_on_export", lang),
                body: translate_and_replace(
                    "export_file_path",
                    &[&path.display().to_string()],
                    lang,
                ),
                kind: NotificationKind::Temporal,
            });
            Ok(())
        }
        Err(e) => {
            error!("Error exporting database: {}", e);
            show_notification(NotificationDefinition {
                title: translate("error_on_export", lang),
                body: e.to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.to_string())
        }
    }
}

/// Returns every translation key resolved to the current UI language, for the frontend's i18n bootstrap.
#[traced_command]
#[tauri::command]
pub fn get_translations(
    settings: State<'_, RwLock<Settings>>,
) -> Result<HashMap<String, String>, String> {
    Ok(TRANSLATIONS
        .keys()
        .map(|k| {
            (
                k.to_string(),
                translate(k, settings.read().unwrap().language),
            )
        })
        .collect())
}
