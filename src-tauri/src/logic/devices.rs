use garmin_tracker_rs_macros::translate;
use nusb::hotplug::HotplugEvent;
use rusqlite_orm::{dao::Repository, database::DATABASE_INST};
use std::sync::{LazyLock, Mutex};
use tokio_stream::StreamExt;

use tauri::{AppHandle, Emitter, async_runtime::JoinHandle};
use tauri_plugin_log::log::{error, info, warn};

use crate::{
    dao::device::{Device, DeviceRepository},
    dto::{
        devices::DeviceListItem,
        notifications::{NotificationDefinition, NotificationKind},
    },
    logic::{notifications::show_notification, sessions::_import_from_device},
    mtp::MTP_CLIENT_INST,
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
        let _ = DATABASE_INST.lock().unwrap().run_in_tx(
            |tx: &mut rusqlite_orm::rusqlite::Transaction<'_>| {
                for device in &cur_dev {
                    if !devices
                        .iter()
                        .any(|e| e.serial_number == device.serial_number)
                    {
                        let enrol_err =
                            match DeviceRepository::select_by_id_in_tx(tx, &device.serial_number) {
                                Ok(None) => DeviceRepository::insert()
                                    .item(Device::from(device))
                                    .execute_in_tx(tx)
                                    .err(),
                                Ok(Some(_)) => None,
                                Err(e) => Some(e),
                            };

                        if let Some(e) = enrol_err {
                            error!(
                                "Error enrolling {} {} ({}): {}",
                                device.manufacturer, device.model, device.serial_number, e
                            )
                        } else {
                            info!(
                                "Connected {} {} ({})",
                                device.manufacturer, device.model, device.serial_number
                            );
                            devices.push(device.clone());

                            let payload: DeviceListItem = device.clone();
                            let _ = app.emit("device_connected", payload);

                            devs_to_sync.push(device.serial_number.clone());
                            show_notification(NotificationDefinition {
                                title: translate!("device_connected"),
                                body: translate!(
                                    "syncing_device",
                                    device.manufacturer,
                                    device.model
                                ),
                                kind: NotificationKind::Temporal,
                            });
                        }
                    }
                }

                Ok(())
            },
        );

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
        let mut imported = 0;
        for dev in devs_to_sync {
            if let Ok(i) = _import_from_device(&dev).await {
                imported += i;
            }
        }
        let _ = app.emit("finish_loading", ());
        if imported > 0 {
            let _ = app.emit("sessions_added", ());
        }
    }
}
