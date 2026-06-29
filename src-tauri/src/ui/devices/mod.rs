pub mod models;

use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tauri_plugin_log::log::info;

use crate::{
    garmin::mtp::MtpClient,
    ui::{
        devices::models::DeviceListItem,
        notifications::{models::NotificationDefinition, show_notification},
    },
};

pub async fn start_device_watcher(app: AppHandle) -> Result<(), String> {
    info!("Starting device monitor...");
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut devices: Vec<DeviceListItem> = Vec::new();
        info!("Device monitor initialized");

        loop {
            if let Ok(cur_dev) = MtpClient::get_connected_devices()
                .await
                .map_err(|e| e.to_string())
            {
                for device in &cur_dev {
                    if !devices
                        .iter()
                        .any(|e| e.serial_number == device.serial_number)
                    {
                        devices.push(device.clone());

                        let payload: DeviceListItem = device.clone();
                        let _ = app.emit("device_connected", payload);

                        info!(
                            "Connected {} {} ({})",
                            device.manufacturer, device.model, device.serial_number
                        );
                        let _ = show_notification(
                            app.clone(),
                            NotificationDefinition {
                                title: "Device connected".to_string(),
                                body: format!("{} {}", device.manufacturer, device.model),
                            },
                        );
                    }
                }

                for device in &devices {
                    if !cur_dev
                        .iter()
                        .any(|d| d.serial_number == device.serial_number)
                    {
                        let payload: DeviceListItem = device.clone();
                        let _ = app.emit("device_disconnected", payload);

                        info!(
                            "Disconnected {} {} ({})",
                            device.manufacturer, device.model, device.serial_number
                        );
                        let _ = show_notification(
                            app.clone(),
                            NotificationDefinition {
                                title: "Device disconnected".to_string(),
                                body: format!("{} {}", device.manufacturer, device.model),
                            },
                        );
                    }
                }

                devices.retain(|d| cur_dev.iter().any(|cd| cd.serial_number == d.serial_number));
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    Ok(())
}
