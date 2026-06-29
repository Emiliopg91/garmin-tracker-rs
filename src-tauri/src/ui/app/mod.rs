use tauri::AppHandle;

use crate::ui::devices::start_device_watcher;
use tauri_plugin_log::log::info;

#[tauri::command]
pub async fn notify_frontend_ready(app: AppHandle) -> Result<(), String> {
    info!("UI readyness notification received");

    start_device_watcher(app.clone()).await?;

    Ok(())
}
