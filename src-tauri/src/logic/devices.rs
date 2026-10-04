use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use nusb::hotplug::HotplugEvent;
use rusqlite_orm::{
    dao::Repository,
    database::DatabasePool,
    types::{value::Value, where_clause::Where},
};
use tokio_stream::StreamExt;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_log::log::{error, info};

use crate::{
    SettingsLock,
    dao::{
        device::{Device, DeviceRepository},
        workout::{self, Workout, WorkoutRepository},
        workout_step::{self, WorkoutStepRepository},
    },
    dto::{
        devices::DeviceListItem,
        notifications::{NotificationDefinition, NotificationKind},
    },
    fit::parser::FitParser,
    logic::{notifications::show_notification, sessions::_import_from_device},
    mtp::MTP_CLIENT_INST,
    utils::{
        constants,
        translations::{translate, translate_and_replace},
    },
};

/// Spawns a background task that watches USB hotplug events and keeps the device list/DB/frontend in sync.
pub fn start_device_watcher(app: AppHandle) {
    info!("Starting device monitor...");
    tauri::async_runtime::spawn(async move {
        let mut devices: Vec<DeviceListItem> = Vec::new();

        match nusb::watch_devices() {
            Ok(w) => {
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
    });
}

/// Diffs the currently connected Garmin devices against `devices`, enrolling new ones in the DB, emitting connect/disconnect events, and triggering auto-sync for newly connected devices.
async fn mtp_dev_check_and_sync(app: AppHandle, devices: &mut Vec<DeviceListItem>) {
    let mut devs_to_sync = Vec::new();
    // Held across device enrolment and workout download; must be dropped before auto-sync,
    // since `_import_from_device` locks the MTP client again.
    let mtp_client = MTP_CLIENT_INST.lock().await;
    let connected = mtp_client
        .get_connected_devices()
        .await
        .map_err(|e| e.to_string());
    if let Ok(cur_dev) = connected {
        let already_known: Vec<String> = devices.iter().map(|d| d.serial_number.clone()).collect();
        let cur_dev_owned = cur_dev.clone();

        let db = app.state::<DatabasePool>();

        let (newly_enrolled, pending_workouts) = db
            .run_in_transaction(|tx| {
                let pending_workouts = WorkoutRepository::select()
                    .columns(&[workout::entity::columns::NAME])
                    .where_(Where::NotInSub(
                        workout::entity::columns::NAME,
                        WorkoutStepRepository::select()
                            .distinct(&[workout_step::entity::columns::WORKOUT])
                            .to_subquery(),
                    ))
                    .fetch_in(tx)?
                    .iter()
                    .filter_map(|r| match r.get(workout::entity::columns::NAME.as_ref()) {
                        Some(Value::Text(name)) => Some(name.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                let mut enrolled = Vec::new();
                for device in &cur_dev_owned {
                    if !already_known.contains(&device.serial_number) {
                        let enrol_err =
                            match DeviceRepository::select_by_id_in(tx, &device.serial_number) {
                                Ok(None) => DeviceRepository::insert()
                                    .item(&mut Device::from(device))
                                    .execute_in(tx)
                                    .err(),
                                Ok(Some(_)) => None,
                                Err(e) => Some(e),
                            };

                        match enrol_err {
                            Some(e) => {
                                error!(
                                    "Error enrolling {} {} ({}): {}",
                                    device.manufacturer, device.model, device.serial_number, e
                                );
                            }
                            None => enrolled.push(device.clone()),
                        }
                    }
                }
                Ok((enrolled, pending_workouts))
            })
            .unwrap_or_default();

        if !pending_workouts.is_empty() {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let src_dir = std::env::temp_dir().join(format!(
                "{}-{}",
                constants::MTP_TMP_DIR_PREFIX,
                now.as_millis()
            ));

            let dst_dir = src_dir.join("Activities");
            for device in &newly_enrolled {
                let _ = mtp_client
                    .download_workouts(&device.serial_number, dst_dir.clone())
                    .await;
            }

            let mut workouts_to_insert = Vec::new();
            if let Ok(read_dir) = fs::read_dir(&dst_dir) {
                for entry in read_dir.flatten() {
                    let file = entry.path();
                    if let Ok(parser) = FitParser::from_file(file)
                        && let Ok(workout) = Workout::try_from(parser)
                        && pending_workouts.contains(&workout.name)
                    {
                        workouts_to_insert.push(workout);
                    }
                }
            }

            let _ = db.run_in_transaction(|tx| {
                let mut insert = WorkoutStepRepository::insert();
                for workout in &mut workouts_to_insert {
                    for step in &mut workout.steps {
                        insert = insert.item(step);
                    }
                }
                Ok(insert.execute_in(tx)?)
            });

            let _ = fs::remove_dir_all(&src_dir);
        }
        drop(mtp_client);

        let (lang, auto_sync) = {
            let settings_state = app.state::<SettingsLock>();
            let settings = settings_state.read().unwrap();
            (settings.language, settings.auto_sync)
        };
        for device in &newly_enrolled {
            info!(
                "Connected {} {} ({})",
                device.manufacturer, device.model, device.serial_number
            );
            devices.push(device.clone());

            let payload: DeviceListItem = device.clone();
            let _ = app.emit("device_connected", payload);

            if auto_sync {
                devs_to_sync.push(device.serial_number.clone());
                show_notification(NotificationDefinition {
                    title: translate("device_connected", lang),
                    body: translate_and_replace(
                        "syncing_device",
                        &[&device.manufacturer, &device.model],
                        lang,
                    ),
                    kind: NotificationKind::Temporal,
                });
            } else {
                show_notification(NotificationDefinition {
                    title: translate("device_connected", lang),
                    body: format!("{} {}", device.manufacturer, device.model),
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
                    title: translate("device_disconnected", lang),
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
            if let Ok(i) = _import_from_device(&app, &dev).await {
                imported += i;
            }
        }
        let _ = app.emit("finish_loading", ());
        if imported > 0 {
            let _ = app.emit("sessions_added", ());
        }
    }
}
