pub mod errors;

use std::path::Path;

use curl_rest::StatusCode;
use serde_json::Value;
use tauri_plugin_log::log::info;
use tokio::{fs, process::Command};

use crate::{rclone::errors::RCloneError, utils::constants};

pub struct RCloneClient;

impl RCloneClient {
    pub async fn configure() -> errors::Result<()> {
        info!("Getting authorization for OneDrive...");
        let output = Command::new("rclone")
            .args(["authorize", "onedrive"])
            .output()
            .await
            .map_err(RCloneError::Authorization)?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if output.status.success() {
            let mut lines = stdout.lines();
            match lines.find(|l| l.starts_with("{")) {
                Some(line) => {
                    let token_line = line;
                    let json_obj = serde_json::from_str::<Value>(token_line)
                        .map_err(RCloneError::BadJsonFormat)?;
                    let bearer_token = json_obj
                        .get("access_token")
                        .unwrap_or_default()
                        .as_str()
                        .unwrap()
                        .to_string();

                    info!("Getting available drives...");
                    let resp = tokio::task::spawn_blocking(move || {
                        curl_rest::Client::default()
                            .get()
                            .header(curl_rest::Header::Authorization(
                                format!("Bearer {bearer_token}").into(),
                            ))
                            .header(curl_rest::Header::Accept("application/json".into()))
                            .send("https://graph.microsoft.com/v1.0/me/drives")
                    })
                    .await
                    .map_err(RCloneError::TaskError)?
                    .map_err(RCloneError::ErrorGettingDrives)?;

                    if resp.status == StatusCode::Ok {
                        let json_obj = serde_json::from_slice::<Value>(resp.body.as_slice())
                            .map_err(RCloneError::BadJsonFormat)?;
                        let drive = json_obj
                            .get("value")
                            .and_then(|v| v.as_array())
                            .into_iter()
                            .flatten()
                            .find_map(|v| {
                                if v.get("driveType").unwrap_or_default() == "personal"
                                    && v.get("name").unwrap_or_default() == "OneDrive"
                                {
                                    Some(v.get("id").unwrap().as_str().unwrap())
                                } else {
                                    None
                                }
                            });

                        if let Some(drive) = drive {
                            info!("Selected drive with ID {drive}");
                            info!("Configuring rclone...");
                            let cfg_output = Command::new("rclone")
                                .args([
                                    "config",
                                    "create",
                                    constants::RCLONE_CONFIG_NAME.as_str(),
                                    "onedrive",
                                    "token",
                                    token_line,
                                    "drive_id",
                                    drive,
                                    "drive_type",
                                    "personal",
                                    "--non-interactive",
                                ])
                                .output()
                                .await
                                .map_err(RCloneError::CommandError)?;

                            if cfg_output.status.success() {
                                info!("Remote 'onedrive' configured");
                                Ok(())
                            } else {
                                Err(RCloneError::BadExitStatus(
                                    cfg_output.status,
                                    String::from_utf8_lossy(&cfg_output.stderr).to_string(),
                                ))
                            }
                        } else {
                            Err(RCloneError::NoDriveFound())
                        }
                    } else {
                        Err(RCloneError::BadResponseStatus(resp.status))
                    }
                }
                None => Err(RCloneError::BadToken()),
            }
        } else {
            Err(RCloneError::BadExitStatus(
                output.status,
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    pub async fn is_configured() -> errors::Result<bool> {
        if let Ok(exist) = fs::try_exists(constants::RCLONE_CONFIG_FILE.as_path()).await
            && exist
        {
            let content = fs::read_to_string(constants::RCLONE_CONFIG_FILE.as_path())
                .await
                .map_err(RCloneError::ReadConfig)?;

            let marker = format!("[{}]", *constants::RCLONE_CONFIG_NAME);
            Ok(content.lines().find(|l| l.trim() == marker).is_some())
        } else {
            Ok(false)
        }
    }

    pub async fn upload<P>(path: P) -> errors::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let result = Command::new("rclone")
            .arg("copy")
            .arg(path.display().to_string())
            .arg(format!(
                "{}:{}",
                *constants::RCLONE_CONFIG_NAME,
                *constants::APP_NAME,
            ))
            .output()
            .await
            .map_err(RCloneError::CommandError)?;

        if result.status.success() {
            Ok(())
        } else {
            Err(RCloneError::BadExitStatus(
                result.status,
                String::from_utf8_lossy(&result.stderr).to_string(),
            ))
        }
    }
}
