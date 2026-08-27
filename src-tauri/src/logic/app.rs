use std::{fs, sync::OnceLock};

use chrono::Local;
use garmin_tracker_rs_macros::{traced_command, translate};
use tauri::{AppHandle, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;
use tokio::sync::RwLock;

use crate::{
    constants,
    dto::{
        app::{AppEnvironment, Settings},
        export::Export,
        notifications::{NotificationDefinition, NotificationKind},
    },
    logic::{devices::start_device_watcher, notifications::show_notification},
};
use tauri_plugin_log::log::{error, info};

pub static SETTINGS_INST: OnceLock<RwLock<Settings>> = OnceLock::new();

#[traced_command]
#[tauri::command]
pub async fn get_settings() -> Settings {
    SETTINGS_INST.get().unwrap().read().await.clone()
}

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
}

#[traced_command]
#[tauri::command]
pub async fn get_environment() -> AppEnvironment {
    if cfg!(debug_assertions) {
        AppEnvironment::Debug
    } else {
        AppEnvironment::Release
    }
}

#[traced_command]
#[tauri::command]
pub async fn update_settings_value(app: AppHandle, name: &str, value: &str) -> Result<(), String> {
    match name {
        crate::dao::settings::settings_keys::AUTO_SYNC => {
            let value = value == "true";
            crate::dao::settings::Settings::set_auto_sync(value).map_err(|e| e.to_string())?;
            SETTINGS_INST.get().unwrap().write().await.auto_sync = value;
        }
        crate::dao::settings::settings_keys::DISTANCE_UNIT => {
            let value = value.try_into()?;
            crate::dao::settings::Settings::set_distance_unit(&value).map_err(|e| e.to_string())?;
            SETTINGS_INST.get().unwrap().write().await.distance_unit = value;
        }
        crate::dao::settings::settings_keys::START_ON_BOOT => {
            let value = value == "true";
            crate::dao::settings::Settings::set_start_on_boot(value).map_err(|e| e.to_string())?;
            if value {
                app.autolaunch().enable()
            } else {
                app.autolaunch().disable()
            }
            .map_err(|e| e.to_string())?;
            SETTINGS_INST.get().unwrap().write().await.start_boot = value;
        }
        crate::dao::settings::settings_keys::WEIGHT_UNIT => {
            let value = value.try_into()?;
            crate::dao::settings::Settings::set_weight_unit(&value).map_err(|e| e.to_string())?;
            SETTINGS_INST.get().unwrap().write().await.weight_unit = value;
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[traced_command]
#[tauri::command]
pub fn export_database() -> Result<(), String> {
    let path = constants::HOME_DIR.join(format!(
        "{}-{}.json",
        *constants::APP_NAME,
        Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
    ));
    info!("Exporting database to {}...", path.display());

    let res = Export::from_database()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        .and_then(|export| {
            export
                .to_json()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
        .and_then(|json| {
            fs::write(&path, json).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        });

    match res {
        Ok(()) => {
            show_notification(NotificationDefinition {
                title: translate!("ok_on_export"),
                body: translate!("export_file_path", path.display()),
                kind: NotificationKind::Temporal,
            });
            Ok(())
        }
        Err(e) => {
            error!("Error exporting database: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_on_export"),
                body: e.to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.to_string())
        }
    }
}
