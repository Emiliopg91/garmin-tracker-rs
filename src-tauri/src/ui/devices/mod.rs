pub mod models;

use tauri::{AppHandle, Emitter};

use crate::{
    garmin::mtp::MtpClient,
    ui::{
        devices::models::DeviceListItem,
        notifications::{models::NotificationDefinition, show_notification},
    },
};

#[tauri::command]
pub async fn start_device_watcher(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn(async move {
        let mut devices: Vec<DeviceListItem> = Vec::new();

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
