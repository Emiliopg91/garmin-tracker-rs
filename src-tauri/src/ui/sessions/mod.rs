pub mod models;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use chrono::{Datelike, Local, TimeZone, Timelike};
use garmin_tracker_rs_macros::{traced_command, translate};
use rfd::AsyncFileDialog;
use tauri_plugin_log::log::{error, info, warn};

use crate::{
    garmin::{
        database::{
            DATABASE_INST,
            dao::{
                Entity,
                device::{DEVICE_COLUMN_LAST_SYNC, DEVICE_COLUMN_SERIAL, Device},
                exercise::{EXERCISE_COLUMN_NAME, Exercise},
                helpers::types::{order_by::OrderBy, where_clause::Where},
                serie::Serie,
                session::Session,
            },
        },
        mtp::MTP_CLIENT_INST,
        parser::load_from_file,
    },
    ui::{
        notifications::{
            models::{NotificationDefinition, NotificationKind},
            show_notification,
        },
        sessions::models::{SessionDetails, SessionListItem, SessionSerie, SessionSeriesUpdate},
    },
};

#[traced_command]
#[tauri::command]
pub fn get_sessions() -> Result<Vec<SessionListItem>, String> {
    info!("Getting sessions list...");
    let res: Result<Vec<SessionListItem>, String> = {
        let sessions = Session::load_from_db(true).map_err(|e| e.to_string())?;

        Ok(sessions
            .into_iter()
            .map(|s| SessionListItem::from(&s))
            .collect::<Vec<_>>())
    };

    match res {
        Ok(l) => {
            info!("Retreived {} sessions", l.len());
            Ok(l)
        }
        Err(e) => {
            error!("Error getting sessions list: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_session_list"),
                body: e.clone(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

#[traced_command]
#[tauri::command]
pub fn get_session_details(timestamp: i64) -> Result<SessionDetails, String> {
    info!(
        "Getting details for session {}",
        Local.timestamp_opt(timestamp, 0).unwrap().to_rfc3339()
    );
    let res = {
        if let Some(session) = Session::find_by_id(timestamp, true).map_err(|e| e.to_string())? {
            let mut details = SessionDetails::from(&session);

            for (exercise, series) in &session.series {
                if !details.exercises.contains(&exercise.name) {
                    details.exercises.push(exercise.name.clone())
                }
                let entry = details.series.entry(exercise.name.clone()).or_default();
                for serie in series {
                    entry.push(SessionSerie::from(serie));
                }
            }

            Ok(details)
        } else {
            Err("Could not find session".to_string())
        }
    };

    match res {
        Ok(l) => {
            info!("Found details for session {} - {}", l.name, l.date);
            Ok(l)
        }
        Err(e) => {
            error!("Error getting session details: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_session_details"),
                body: e.clone(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

#[traced_command]
#[tauri::command]
pub fn save_session_changes(details: SessionSeriesUpdate) -> Result<(), String> {
    info!(
        "Saving changes on session {}...",
        Local
            .timestamp_opt(details.timestamp, 0)
            .unwrap()
            .to_rfc3339()
    );
    let res: Result<(), String> = {
        let mut to_update = Vec::new();
        for serie in details.series {
            let db_serie =
                Serie::select_by_id(details.timestamp, serie.idx).map_err(|e| e.to_string())?;
            if let Some(mut db_serie) = db_serie {
                db_serie.reps = serie.reps;
                db_serie.weight = serie.weight;
                to_update.push(db_serie);
            }
        }
        let exercises = Exercise::select()
            .order_by(OrderBy::Asc(EXERCISE_COLUMN_NAME))
            .fetch()
            .map_err(|e| e.to_string())?;

        let mut db = DATABASE_INST.lock().map_err(|e| e.to_string())?;
        db.run_in_transaction(move |tx| {
            for to_upd in &to_update {
                to_upd.update_reps_and_weight(tx)?;
            }
            for exer in &exercises {
                Serie::update_pr(tx, &exer.category, exer.id);
            }
            Ok(())
        })
        .map_err(|e| e.to_string())?;

        Ok(())
    };

    match res {
        Ok(l) => {
            info!("Session updated succesfully");
            show_notification(NotificationDefinition {
                title: translate!("ok_update_session"),
                body: "".to_string(),
                kind: NotificationKind::Temporal,
            });

            Ok(l)
        }
        Err(e) => {
            error!("Error updating session: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_update_session"),
                body: e.clone(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

#[traced_command]
#[tauri::command]
pub async fn import_from_file() -> Result<u16, String> {
    info!("Starting import from file/s...");

    info!("Waiting for user to select files...");
    let selection = AsyncFileDialog::new()
        .set_directory(std::env::current_dir().unwrap())
        .set_can_create_directories(false)
        .add_filter("Garmin FIT file", &["fit"])
        .pick_files()
        .await;

    match selection {
        Some(files) => {
            let files = files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect::<Vec<PathBuf>>();
            info!("Selected files: {:?}", files);
            import_file_list(&files, true)
        }
        None => {
            info!("No file was selected");
            Ok(0)
        }
    }
}

#[traced_command]
#[tauri::command]
pub async fn import_from_device(serial: &str) -> Result<u16, String> {
    info!("Starting import from device with S/N {}", serial);
    let mut latest_date = "2026-06-08-00-00-00".to_string();

    if let Ok(devs) = Device::select_by_id(serial.to_string())
        && let Some(dev) = devs.into_iter().next()
        && let Some(latest) = dev.last_sync
    {
        let latest = Local.timestamp_opt(latest, 0).unwrap();
        latest_date = format!(
            "{:04}-{:02}-{:02}-{:02}-{:02}-{:02}",
            latest.year(),
            latest.month(),
            latest.day(),
            latest.hour(),
            latest.minute(),
            latest.second(),
        );
    }

    info!(
        "Fetching from device activity files after {}...",
        latest_date
    );
    let activities = MTP_CLIENT_INST
        .lock()
        .await
        .download_activities_since(serial, latest_date)
        .await
        .map_err(|e| e.to_string())?;

    if !activities.is_empty() {
        info!("Fetched {} activity files", activities.len());

        match import_file_list(&activities, false) {
            Ok(inserted) => {
                if let Ok(devs) = Device::select_by_id(serial.to_string())
                    && let Some(dev) = devs.into_iter().next()
                {
                    let _ = Device::update()
                        .set(
                            DEVICE_COLUMN_LAST_SYNC,
                            Some(Local::now().timestamp()).into(),
                        )
                        .where_(Where::Eq(DEVICE_COLUMN_SERIAL, dev.serial.into()))
                        .execute();
                }
                Ok(inserted)
            }
            Err(e) => Err(e),
        }
    } else {
        Ok(0)
    }
}

fn import_file_list<F>(files: &[F], error_on_duplicate: bool) -> Result<u16, String>
where
    F: AsRef<Path>,
{
    let mut success = 0_u16;

    let mut latest: Option<i64> = None;
    for file in files {
        info!("Importing file {}", file.as_ref().display());
        let res = match load_from_file(file.as_ref()) {
            Ok(session) => {
                let found = Session::find_by_id(session.date, false)
                    .map(|opt| opt.is_some())
                    .unwrap_or(false);

                if !found {
                    let mut db = DATABASE_INST.lock().unwrap();
                    if let Err(e) = db.run_in_transaction(|tx| {
                        Session::insert()
                            .item(session.clone())
                            .execute_in_transaction(tx)?;

                        let mut insert = Exercise::insert().or_ignore(true);
                        let mut seen = HashSet::new();
                        for exercise in session.series.iter().map(|e| e.0) {
                            if seen.insert(exercise.clone()) {
                                insert = insert.item(exercise.clone());
                            }
                        }
                        insert.execute_in_transaction(tx)?;

                        let mut insert = Serie::insert().or_ignore(true);
                        for series in session.series.iter().map(|e| e.1) {
                            for serie in series {
                                insert = insert.item(serie.clone());
                            }
                        }
                        insert.execute_in_transaction(tx)?;

                        Ok(())
                    }) {
                        Err(format!("Error persisting session: {}", e))
                    } else {
                        success += 1;
                        latest = if let Some(latest_v) = latest {
                            if session.date > latest_v {
                                Some(session.date)
                            } else {
                                latest
                            }
                        } else {
                            Some(session.date)
                        };
                        Ok("Session imported succesfully".to_string())
                    }
                } else {
                    let msg = format!("Session with date {} already exists", session.format_date());
                    if error_on_duplicate {
                        Err(format!(
                            "Session with date {} already exists",
                            session.format_date()
                        ))
                    } else {
                        warn!("{}", msg);
                        Err("".to_string())
                    }
                }
            }
            Err(e) => Err(format!("Error parsing session: {}", e)),
        };

        match res {
            Ok(msg) => {
                info!("  {}", msg);

                show_notification(NotificationDefinition {
                    title: format!("{}", file.as_ref().file_name().unwrap().display()),
                    body: msg,
                    kind: NotificationKind::Temporal,
                });
            }
            Err(e) if !e.is_empty() => {
                error!("  {}", e);

                show_notification(NotificationDefinition {
                    title: format!("{}", file.as_ref().file_name().unwrap().display()),
                    body: e,
                    kind: NotificationKind::Persistant,
                });
            }
            _ => {}
        }
    }

    Ok(success)
}
