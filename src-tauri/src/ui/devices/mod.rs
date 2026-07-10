pub mod models;

use std::{
    sync::{LazyLock, Mutex},
    time::Duration,
};

use tauri::{AppHandle, Emitter, async_runtime::JoinHandle};
use tauri_plugin_log::log::{info, warn};

use crate::{
    garmin::{database::dao::device::Device, mtp::MTP_CLIENT_INST},
    ui::{
        devices::models::DeviceListItem,
        notifications::{
            models::{NotificationDefinition, NotificationKind},
            show_notification,
        },
        translations::{TRANSLATOR_INST, translation_keys::TranslationKeys},
    },
};

static DEVICE_WATCHER: LazyLock<Mutex<Option<JoinHandle<()>>>> = LazyLock::new(|| Mutex::new(None));

pub async fn start_device_watcher(app: AppHandle) {
    let mut watcher = DEVICE_WATCHER.lock().unwrap();
    if watcher.is_some() {
        warn!("Device monitor already running")
    } else {
        info!("Starting device monitor...");
        *watcher = Some(tauri::async_runtime::spawn(async move {
            let mut devices: Vec<DeviceListItem> = Vec::new();
            info!("Device monitor initialized");

            tokio::time::sleep(Duration::from_secs(1)).await;
            loop {
                if let Ok(cur_dev) = MTP_CLIENT_INST
                    .lock()
                    .await
                    .get_connected_devices()
                    .await
                    .map_err(|e| e.to_string())
                {
                    for device in &cur_dev {
                        if !devices
                            .iter()
                            .any(|e| e.serial_number == device.serial_number)
                        {
                            devices.push(device.clone());

                            let device_dao = Device::from(device);
                            let _ = device_dao.insert();

                            let payload: DeviceListItem = device.clone();
                            let _ = app.emit("device_connected", payload);

                            info!(
                                "Connected {} {} ({})",
                                device.manufacturer, device.model, device.serial_number
                            );
                            show_notification(NotificationDefinition {
                                title: TRANSLATOR_INST.translate(TranslationKeys::DEVICE_CONNECTED),
                                body: format!("{} {}", device.manufacturer, device.model),
                                kind: NotificationKind::Temporal,
                            });
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
                            show_notification(NotificationDefinition {
                                title: TRANSLATOR_INST
                                    .translate(TranslationKeys::DEVICE_DISCONNECTED),
                                body: format!("{} {}", device.manufacturer, device.model),
                                kind: NotificationKind::Temporal,
                            });
                        }
                    }

                    devices
                        .retain(|d| cur_dev.iter().any(|cd| cd.serial_number == d.serial_number));
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }));
    }
}
