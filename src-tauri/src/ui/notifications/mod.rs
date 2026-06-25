pub mod models;

use tauri::AppHandle;
use tauri_plugin_log::log::error;
use tauri_plugin_notification::NotificationExt;

use crate::{constants::ICON_PATH, ui::notifications::models::NotificationDefinition};

pub fn show_notification(
    app: AppHandle,
    notification: NotificationDefinition,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(&notification.title)
        .body(&notification.body)
        .icon(ICON_PATH.as_str())
        .show()
        .map_err(|e| {
            error!("Could not send notification: {}", e);
            e.to_string()
        })?;

    Ok(())
}
