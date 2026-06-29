use std::{collections::HashMap, process::Command, time::Duration};

use semver::Version;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::{
    constants,
    ui::{
        devices::start_device_watcher,
        notifications::{models::NotificationDefinition, show_notification},
    },
};
use tauri_plugin_http::reqwest;
use tauri_plugin_log::log::{debug, error, info};

#[tauri::command]
pub async fn notify_frontend_ready(app: AppHandle) -> Result<(), String> {
    info!("UI readyness notification received");

    start_device_watcher(app.clone()).await?;
    update_watcher(app.clone()).await;

    Ok(())
}

async fn update_watcher(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let current_version = Version::parse(constants::APP_VERSION.as_str()).unwrap();
            debug!("Looking for updates...");

            match reqwest::get(constants::AUR_RPC_URL.clone()).await {
                Ok(response) => match response.text().await {
                    Ok(response_body) => {
                        match serde_json::from_str::<HashMap<String, Value>>(&response_body) {
                            Ok(parsed_response) => {
                                if let Some(Value::Array(results)) = parsed_response.get("results")
                                    && let Some(Value::Object(result)) = results.first()
                                {
                                    if let Some(Value::String(version)) = result.get("Version") {
                                        let version = version.split("-").next().unwrap();

                                        match Version::parse(version) {
                                            Ok(latest_version) => {
                                                debug!("Latest version: {}", latest_version);
                                                if latest_version > current_version {
                                                    info!("New update found: {}", latest_version);
                                                    let _ = show_notification(
                                                        app.clone(),
                                                        NotificationDefinition {
                                                            title: "New update available"
                                                                .to_string(),
                                                            body: format!(
                                                                "v{} available, update the application to get latests features and improvements",
                                                                version
                                                            ),
                                                        },
                                                    );
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
    });
}

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
            error!("Error on URL opening request: {}", e);
            Err(e)
        }
    }
}
