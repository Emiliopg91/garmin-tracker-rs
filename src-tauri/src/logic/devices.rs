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
        session::{self, SessionRepository},
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
    let mtp_client = MTP_CLIENT_INST.lock().await;
    let connected = mtp_client
        .get_connected_devices()
        .await
        .map_err(|e| e.to_string());
    drop(mtp_client);

    if let Ok(cur_dev) = connected {
        let already_known: Vec<String> = devices.iter().map(|d| d.serial_number.clone()).collect();

        let db = app.state::<DatabasePool>();

        let newly_enrolled = db
            .run_in_transaction(|tx| {
                let mut enrolled = Vec::new();
                for device in &cur_dev {
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

                Ok(enrolled)
            })
            .unwrap_or_default();

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

            download_pending_workouts(&app, &db, device).await;

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

/// Downloads from `device` the step definitions of workouts that have sessions recorded on it but no steps stored yet.
async fn download_pending_workouts(app: &AppHandle, db: &DatabasePool, device: &DeviceListItem) {
    let pending_workouts = match db.run_in_connection(|conn| {
        let rows = WorkoutRepository::select()
            .columns(&[workout::entity::columns::NAME])
            .where_(Where::And(vec![
                Where::NotInSub(
                    workout::entity::columns::NAME,
                    WorkoutStepRepository::select()
                        .distinct(&[workout_step::entity::columns::WORKOUT])
                        .to_subquery(),
                ),
                Where::InSub(
                    workout::entity::columns::NAME,
                    SessionRepository::select()
                        .distinct(&[session::entity::columns::WORKOUT])
                        .where_(Where::Eq(
                            session::entity::columns::DEVICE,
                            device.serial_number.clone().into(),
                        ))
                        .to_subquery(),
                ),
            ]))
            .fetch_in(conn)?;
        Ok(rows)
    }) {
        Ok(rows) => rows
            .iter()
            .filter_map(|r| match r.get(workout::entity::columns::NAME.as_ref()) {
                Some(Value::Text(name)) => Some(name.clone()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            error!(
                "Error listing pending workouts for {}: {e}",
                device.serial_number
            );
            return;
        }
    };
    if pending_workouts.is_empty() {
        return;
    }

    let dst_dir = std::env::temp_dir().join(format!(
        "{}-{}",
        constants::MTP_TMP_DIR_PREFIX,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));

    // Locked only for the download, so a concurrent sync is not held back by DB work.
    let downloaded = MTP_CLIENT_INST
        .lock()
        .await
        .download_workouts(&device.serial_number, dst_dir.clone())
        .await;

    match downloaded {
        Ok(()) => {
            let mut workouts_to_insert = Vec::new();
            if let Ok(read_dir) = fs::read_dir(&dst_dir) {
                for entry in read_dir.flatten() {
                    if let Ok(parser) = FitParser::from_file(entry.path())
                        && let Ok(workout) = Workout::try_from(parser)
                        && !workout.steps.is_empty()
                        && pending_workouts.contains(&workout.name)
                    {
                        workouts_to_insert.push(workout);
                    }
                }
            }

            let mut insert = WorkoutStepRepository::insert();
            let mut workout_names: Vec<String> = Vec::new();
            for workout in &mut workouts_to_insert {
                for step in &mut workout.steps {
                    insert = insert.item(step);
                }
                workout_names.push(workout.name.clone());
            }

            if !workout_names.is_empty() {
                if let Err(e) = insert.execute(db) {
                    error!(
                        "Error storing workout steps from {}: {e}",
                        device.serial_number
                    );
                } else {
                    let _ = app.emit("added_workout_steps", workout_names);
                }
            }
        }
        Err(e) => error!(
            "Error downloading workouts from {}: {e}",
            device.serial_number
        ),
    }

    let _ = fs::remove_dir_all(&dst_dir);
}
