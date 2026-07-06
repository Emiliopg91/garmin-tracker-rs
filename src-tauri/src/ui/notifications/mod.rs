pub mod models;

use std::process::Command;

use tauri_plugin_log::log::error;

use crate::{
    constants::{APP_TITLE, ICON_PATH},
    ui::notifications::models::NotificationDefinition,
};

pub fn show_notification(notification: NotificationDefinition) {
    let icon_param = &format!("--icon={}", *ICON_PATH);
    let expire_param = &format!("--expire-time={}", notification.kind.get_timeout());
    let app_name_param = &format!("--app-name={}", APP_TITLE);

    let args = vec![
        notification.title.as_str(),
        notification.body.as_str(),
        icon_param,
        expire_param,
        app_name_param,
    ];

    match Command::new("notify-send").args(args).status() {
        Ok(status) => {
            if !status.success() {
                error!(
                    "Error while showing notification, exit code: {:?}",
                    status.code()
                );
            }
        }
        Err(e) => {
            error!("Error while showing notification: {}", e);
        }
    }
}
