pub mod models;

use std::{
    collections::HashMap, 
    process::Command,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use garmin_tracker_rs_macros::{traced_command, translate};
use semver::Version;
use serde_json::Value;
use tauri::{AppHandle, Emitter, WebviewWindow, async_runtime::JoinHandle};

use crate::{
    constants,
    ui::{
        app::models::AppEnvironment,
        devices::start_device_watcher,
        notifications::{
            models::{NotificationDefinition, NotificationKind},
            show_notification,
        },
    },
};
use tauri_plugin_log::log::{debug, error, info, warn};

static UPDATE_WATCHER: LazyLock<Mutex<Option<JoinHandle<()>>>> = LazyLock::new(|| Mutex::new(None));

#[traced_command]
#[tauri::command]
pub async fn notify_frontend_ready(app: AppHandle, webview_window: WebviewWindow) {
    info!("UI ready");

    update_watcher(app.clone()).await;
    start_device_watcher(app.clone()).await;

    tokio::time::sleep(Duration::from_millis(100)).await;
    info!("Full application ready");

    info!("Showing up main window...");
    std::thread::spawn(move || {
        std::thread::sleep(tokio::time::Duration::from_millis(200));
        let _ = webview_window.set_title(&format!(
            "{} v{}",
            webview_window.title().unwrap(),
            *constants::APP_VERSION
        ));
        let _ = webview_window.show();
    });
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

async fn update_watcher(app: AppHandle) {
    let mut watcher = UPDATE_WATCHER.lock().unwrap();
    if watcher.is_some() {
        warn!("Update watcher already running")
    } else {
        info!("Starting update watcher...");
        *watcher = Some(tauri::async_runtime::spawn(async move {
            info!("Update watcher initialized");
            tokio::time::sleep(Duration::from_secs(1)).await;
            loop {
                let current_version = Version::parse(constants::APP_VERSION.as_str()).unwrap();
                debug!("Looking for updates...");

                match ureq::get(constants::AUR_RPC_URL.clone()).call() {
                    Ok(response) => match response.into_body().read_to_string() {
                        Ok(response_body) => {
                            match serde_json::from_str::<HashMap<String, Value>>(&response_body) {
                                Ok(parsed_response) => {
                                    if let Some(Value::Array(results)) =
                                        parsed_response.get("results")
                                        && let Some(Value::Object(result)) = results.first()
                                    {
                                        if let Some(Value::String(version)) = result.get("Version")
                                        {
                                            let version = version.split("-").next().unwrap();

                                            match Version::parse(version) {
                                                Ok(latest_version) => {
                                                    debug!("Latest version: {}", latest_version);
                                                    if latest_version > current_version {
                                                        info!(
                                                            "New update found: {}",
                                                            latest_version
                                                        );
                                                        show_notification(NotificationDefinition {
                                                            title: translate!("new_update_title"),
                                                            body: translate!(
                                                                    "new_update_body",
                                                                    version,
                                                                ),
                                                            kind: NotificationKind::Temporal,
                                                        });
                                                        let version: String = version.to_string();
                                                        let _ = app.emit_str(
                                                            "update_available",
                                                            format!("\"{}\"", version),
                                                        );
                                                        break;
                                                    }
                                                }
                                                Err(e) => {
                                                    debug!("Error parsing version from AUR: {}", e)
                                                }
                                            }
                                        } else {
                                            debug!("No version found from AUR RPC")
                                        }
                                    } else {
                                        debug!("No results found from AUR RPC")
                                    }
                                }
                                Err(e) => {
                                    debug!("Error parsing response from AUR: {}", e)
                                }
                            }
                        }
                        Err(e) => debug!("{}", e),
                    },
                    Err(e) => {
                        debug!("{}", e)
                    }
                }
                debug!("No updates found");
                tokio::time::sleep(Duration::from_hours(1)).await;
            }
        }));
    }
}

#[traced_command]
#[tauri::command]
pub fn open_version_changelog(version: &str) -> Result<(), String> {
    info!("Opening changelog for version {}...", version);
    match Command::new("xdg-open")
        .arg(format!(
            "https://github.com/Emiliopg91/garmin-tracker-rs/releases/tag/{}",
            version
        ))
        .output()
        .map(|_| ())
        .map_err(|e| e.to_string())
    {
        Ok(_) => {
            info!("Opening request succesful");
            Ok(())
        }
        Err(e) => {
            error!("Error opening URL: {}", e);
            Err(e)
        }
    }
}
