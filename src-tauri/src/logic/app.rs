use garmin_tracker_rs_macros::traced_command;
use tauri::{AppHandle, WebviewWindow};

use crate::{constants, dto::app::AppEnvironment, logic::devices::start_device_watcher};
use tauri_plugin_log::log::info;

#[traced_command]
#[tauri::command]
pub async fn notify_frontend_ready(app: AppHandle, webview_window: WebviewWindow) {
    info!("UI ready");

    start_device_watcher(app.clone()).await;

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
