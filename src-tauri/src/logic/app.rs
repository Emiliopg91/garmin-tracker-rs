use std::{collections::HashMap, fs, thread, time::Duration};

use chrono::Local;
use garmin_tracker_rs_macros::traced_command;
use rusqlite_orm::database::DatabasePool;
use semver::Version;
use serde_json::Value;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    SettingsLock, constants,
    dao::settings::settings_keys::{
        AUTO_SYNC, DISTANCE_UNIT, LANGUAGE, START_ON_BOOT, WEIGHT_UNIT,
    },
    dto::{
        app::{AppEnvironment, Settings},
        export::Export,
        notifications::{NotificationDefinition, NotificationKind},
    },
    logic::{
        devices::start_device_watcher, notifications::show_notification, report_error,
        sessions::update_pending_geolocation,
    },
    utils::translations::{Languages, TRANSLATIONS, translate, translate_and_replace},
};
use tauri_plugin_log::log::{debug, info};

/// Returns the current in-memory app settings.
#[traced_command]
#[tauri::command]
pub async fn get_settings(settings: State<'_, SettingsLock>) -> Result<Settings, String> {
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

    let app_cl = app.clone();
    std::thread::spawn(move || {
        let db = app_cl.state::<DatabasePool>();
        update_pending_geolocation(&app_cl, &db);
    });

    let app_cl = app.clone();
    std::thread::spawn(|| check_for_update(app_cl));
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
pub async fn update_settings_value(app: AppHandle, name: &str, value: &str) -> Result<(), String> {
    let name = name.to_string();
    let value = value.to_string();

    let lang = app.state::<SettingsLock>().read().unwrap().language;

    let res = tokio::task::spawn_blocking(move || {
        let database = app.state::<DatabasePool>();
        let settings = app.state::<SettingsLock>();
        match name.as_str() {
            AUTO_SYNC => {
                let value = value == "true";
                crate::dao::settings::Settings::set_auto_sync(&database, value)
                    .map_err(|e| e.to_string())?;
                settings.write().unwrap().auto_sync = value;
            }
            DISTANCE_UNIT => {
                let value = value.as_str().try_into()?;
                crate::dao::settings::Settings::set_distance_unit(&database, &value)
                    .map_err(|e| e.to_string())?;
                settings.write().unwrap().distance_unit = value;
            }
            START_ON_BOOT => {
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
            WEIGHT_UNIT => {
                let value = value.as_str().try_into()?;
                crate::dao::settings::Settings::set_weight_unit(&database, &value)
                    .map_err(|e| e.to_string())?;
                settings.write().unwrap().weight_unit = value;
            }
            LANGUAGE => {
                let value = Languages::from_name(&value);
                crate::dao::settings::Settings::set_language(&database, &value)
                    .map_err(|e| e.to_string())?;
                settings.write().unwrap().language = value;
            }
            _ => unreachable!(),
        }

        Ok(settings.read().unwrap().language)
    })
    .await
    .map_err(|e| e.to_string())
    .flatten();

    match res {
        Ok(lang) => {
            show_notification(NotificationDefinition {
                title: translate("ok_update_settings", lang),
                body: String::new(),
                kind: NotificationKind::Temporal,
            });
            Ok(())
        }
        Err(e) => Err(report_error(
            e,
            lang,
            "error_update_settings",
            "Error updating settings",
        )),
    }
}

/// Exports the whole database to a timestamped JSON file in the user's home directory and notifies on success/failure.
#[traced_command]
#[tauri::command]
pub fn export_database(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
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
        Err(e) => Err(report_error(
            e,
            lang,
            "error_on_export",
            "Error exporting database",
        )),
    }
}

/// Returns every translation key resolved to the current UI language, for the frontend's i18n bootstrap.
#[traced_command]
#[tauri::command]
pub fn get_translations(
    settings: State<'_, SettingsLock>,
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

fn check_for_update(app: AppHandle) {
    thread::sleep(Duration::from_secs(5));

    loop {
        info!("Looking for updates...");

        let result = curl_rest::Client::with_user_agent("garmin-tracker-rs")
            .get()
            .header(curl_rest::Header::Accept(
                "application/vnd.github+json".into(),
            ))
            .send(constants::URL);

        match result {
            Ok(response) => {
                if response.status == curl_rest::StatusCode::Ok {
                    match serde_json::from_slice::<Value>(&response.body) {
                        Ok(response_json) => match response_json.get("tag_name") {
                            Some(tag_name) => match Version::parse(tag_name.as_str().unwrap()) {
                                Ok(version) => {
                                    if version > *constants::APP_SEM_VERSION {
                                        info!("Update {} found!", version);
                                        let lang =
                                            app.state::<SettingsLock>().read().unwrap().language;
                                        show_notification(NotificationDefinition {
                                            title: translate("new_update_title", lang),
                                            body: translate_and_replace(
                                                "new_update_body",
                                                &[&version.to_string()],
                                                lang,
                                            ),
                                            kind: NotificationKind::Persistant,
                                        });
                                        break;
                                    } else {
                                        info!("No update found");
                                    }
                                }
                                Err(e) => {
                                    debug!("Error parsing version: {}", e)
                                }
                            },
                            None => {
                                debug!("No tag name found")
                            }
                        },
                        Err(e) => {
                            debug!("Invalid parsing response: {}", e)
                        }
                    }
                } else {
                    debug!("Invalid response status: {}", response.status,)
                }
            }
            Err(e) => {
                debug!("Error sending request to GitHub: {}", e)
            }
        }
        thread::sleep(Duration::from_hours(1));
    }
}
