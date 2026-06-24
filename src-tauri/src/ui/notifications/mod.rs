pub mod models;

use std::sync::LazyLock;

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::ui::notifications::models::NotificationDefinition;

static ICON_PATH: LazyLock<String> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    return std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join("../../icons/icon.png")
        .display()
        .to_string();

    #[cfg(not(debug_assertions))]
    return "/usr/share/icons/hicolor/128x128/apps/garmin-tracker-rs.png".to_string();
});

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
            eprintln!("Could not send notification: {}", e);
            e.to_string()
        })?;

    Ok(())
}
