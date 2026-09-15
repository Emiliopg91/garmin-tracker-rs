pub mod errors;

use std::{ffi::OsStr, path::Path, process::Output};

use curl_rest::StatusCode;
use serde_json::Value;
use tauri_plugin_log::log::info;
use tokio::{fs, process::Command};

use crate::{rclone::errors::RCloneError, utils::constants};

pub struct RCloneClient;

impl RCloneClient {
    async fn run_rclone<I, S>(
        args: I,
        io_err: fn(std::io::Error) -> RCloneError,
    ) -> errors::Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("rclone")
            .args(args)
            .output()
            .await
            .map_err(io_err)?;

        if output.status.success() {
            Ok(output)
        } else {
            Err(RCloneError::BadExitStatus(
                output.status,
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    pub async fn configure() -> errors::Result<()> {
        info!("Getting authorization for OneDrive...");
        let output =
            Self::run_rclone(["authorize", "onedrive"], RCloneError::Authorization).await?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let token_line = stdout
            .lines()
            .find(|l| l.starts_with("{"))
            .ok_or_else(RCloneError::BadToken)?;
        let json_obj =
            serde_json::from_str::<Value>(token_line).map_err(RCloneError::BadJsonFormat)?;
        let bearer_token = json_obj
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or(RCloneError::MissingJsonField("access_token"))?
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

        if resp.status != StatusCode::Ok {
            return Err(RCloneError::BadResponseStatus(resp.status));
        }

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
                    v.get("id").and_then(|id| id.as_str())
                } else {
                    None
                }
            })
            .ok_or_else(RCloneError::NoDriveFound)?;

        info!("Selected drive with ID {drive}");
        info!("Configuring rclone...");
        Self::run_rclone(
            [
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
            ],
            RCloneError::CommandError,
        )
        .await?;

        info!("Remote 'onedrive' configured");
        Ok(())
    }

    pub async fn is_configured() -> errors::Result<bool> {
        if let Ok(exist) = fs::try_exists(constants::RCLONE_CONFIG_FILE.as_path()).await
            && exist
        {
            let content = fs::read_to_string(constants::RCLONE_CONFIG_FILE.as_path())
                .await
                .map_err(RCloneError::ReadConfig)?;

            let marker = format!("[{}]", *constants::RCLONE_CONFIG_NAME);
            Ok(content.lines().any(|l| l.trim() == marker))
        } else {
            Ok(false)
        }
    }

    pub async fn upload<P>(path: P) -> errors::Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let remote = format!(
            "{}:{}",
            *constants::RCLONE_CONFIG_NAME,
            *constants::APP_NAME,
        );

        Self::run_rclone(
            [OsStr::new("copy"), path.as_os_str(), OsStr::new(&remote)],
            RCloneError::CommandError,
        )
        .await?;

        Ok(())
    }
}
