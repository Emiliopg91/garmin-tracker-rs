use std::ffi::OsStr;

use tauri_plugin_log::log::info;

use crate::rclone::{
    RCloneClient, RCloneTrait,
    errors::{self, RCloneError},
    providers::DropBox,
};

impl RCloneTrait for RCloneClient<DropBox> {
    fn get_provider_suffix() -> String {
        "dropbox".to_string()
    }

    async fn configure(&self) -> errors::Result<()> {
        info!("Getting authorization for Dropbox...");

        Self::run_rclone(
            &[
                OsStr::new("config"),
                OsStr::new("create"),
                OsStr::new(&Self::get_provider_key()),
                OsStr::new("dropbox"),
            ],
            RCloneError::CommandError,
        )
        .await?;

        info!("Remote 'dropbox' configured");
        Ok(())
    }
}
