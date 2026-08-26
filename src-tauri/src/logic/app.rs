use std::{fs, sync::OnceLock};

use garmin_tracker_rs_macros::traced_command;
use tauri::{AppHandle, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;
use tokio::sync::RwLock;

use crate::{
    constants,
    dto::{
        app::{AppEnvironment, Settings},
        export::Export,
    },
    logic::devices::start_device_watcher,
};
use tauri_plugin_log::log::info;

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
    let export = Export::from_database().map_err(|e| e.to_string())?;
    let json = export.to_json().map_err(|e| e.to_string())?;
    fs::write(constants::HOME_DIR.join(format!("{}.json",*constants::APP_NAME)), json).map_err(|e| e.to_string())?;

    Ok(())
}
