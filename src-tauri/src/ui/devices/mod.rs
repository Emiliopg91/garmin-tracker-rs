pub mod models;

use garmin_tracker_rs_macros::translate;
use nusb::hotplug::HotplugEvent;
use std::sync::{LazyLock, Mutex};
use tokio_stream::StreamExt;

use tauri::{AppHandle, Emitter, async_runtime::JoinHandle};
use tauri_plugin_log::log::{error, info, warn};

use crate::{
    garmin::{
        database::dao::{Entity, device::Device},
        mtp::MTP_CLIENT_INST,
    },
    ui::{
        devices::models::DeviceListItem,
        notifications::{
            models::{NotificationDefinition, NotificationKind},
            show_notification,
        },
        sessions::_import_from_device,
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

            match nusb::watch_devices() {
                Ok(w) => {
                    info!("Device monitor initialized");

                    mtp_dev_check_and_sync(app.clone(), &mut devices).await;
                    let mut watch = w;
                    while let Some(event) = watch.next().await {
                        match event {
                            HotplugEvent::Connected(_) | HotplugEvent::Disconnected(_) => {
                                mtp_dev_check_and_sync(app.clone(), &mut devices).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Could not initialize device monitor: {e}");
                }
            };
        }));
    }
}

async fn mtp_dev_check_and_sync(app: AppHandle, devices: &mut Vec<DeviceListItem>) {
    let mut devs_to_sync = Vec::new();
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

                if let Ok(None) = Device::select_by_id(&device.serial_number) {
                    let _ = Device::insert().item(Device::from(device)).execute();
                }

                let payload: DeviceListItem = device.clone();
                let _ = app.emit("device_connected", payload);

                info!(
                    "Connected {} {} ({})",
                    device.manufacturer, device.model, device.serial_number
                );
                devs_to_sync.push(device.serial_number.clone());
                show_notification(NotificationDefinition {
                    title: translate!("device_connected"),
                    body: translate!("syncing_device", device.manufacturer, device.model),
                    kind: NotificationKind::Temporal,
                });
            }
        }

        for device in devices.iter() {
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
                    title: translate!("device_disconnected"),
                    body: format!("{} {}", device.manufacturer, device.model),
                    kind: NotificationKind::Temporal,
                });
            }
        }

        devices.retain(|d| cur_dev.iter().any(|cd| cd.serial_number == d.serial_number));
    }
    if !devs_to_sync.is_empty() {
        let _ = app.emit("start_loading", ());
        for dev in devs_to_sync {
            let _ = _import_from_device(&dev).await;
        }
        let _ = app.emit("finish_loading", ());
    }
}
