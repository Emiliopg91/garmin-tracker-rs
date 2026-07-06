pub mod models;
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::log::{error, info};

use crate::{
    garmin::{
        database::{
            DATABASE_INST,
            dao::{device::Device, exercise::Exercise, serie::Serie, session::Session},
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
                title: "Error getting sessions list".to_string(),
                body: e.clone(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

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
                title: "Error getting session details".to_string(),
                body: e.clone(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

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
            let db_serie = Serie::load_for_session_and_idx(details.timestamp, serie.idx)
                .map_err(|e| e.to_string())?;
            if let Some(mut db_serie) = db_serie {
                db_serie.reps = serie.reps;
                db_serie.weight = serie.weight;
                to_update.push(db_serie);
            }
        }
        let exercises = Exercise::load_from_db().map_err(|e| e.to_string())?;

        let mut db = DATABASE_INST.lock().map_err(|e| e.to_string())?;
        db.run_in_transaction(move |tx| {
            for to_upd in &to_update {
                to_upd.update_serie(tx);
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
                title: "Session updated succesfully".to_string(),
                body: "".to_string(),
                kind: NotificationKind::Temporal,
            });

            Ok(l)
        }
        Err(e) => {
            error!("Error updating session: {}", e);
            show_notification(NotificationDefinition {
                title: "Error updating session".to_string(),
                body: e.clone(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn import_from_file(app: AppHandle) -> Result<u16, String> {
    info!("Starting import from file/s...");
    let (tx, rx) = mpsc::channel();

    info!("Waiting for user to select files...");
    app.dialog()
        .file()
        .add_filter("Garmin FIT file", &["fit"])
        .pick_files(move |file| {
            if let Some(file) = file {
                let _ = tx.send(file);
            }
        });

    let res = match rx.recv() {
        Ok(files) => {
            let files = files
                .iter()
                .filter_map(|f| f.as_path().map(|p| p.to_path_buf()))
                .collect::<Vec<PathBuf>>();
            info!("Selected files: {:?}", files);
            import_file_list(&files)
        }
        Err(_) => {
            info!("No file was selected");
            Ok((0, None))
        }
    };

    match res {
        Ok((inserted, _)) => Ok(inserted),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn import_from_device(serial: &str) -> Result<u16, String> {
    info!("Starting import from device with S/N {}", serial);
    let mut latest_date = "2026-06-08-00-00-00".to_string();
    if let Ok(Some(device)) = Device::find_by_id(serial)
        && let Some(latest) = device.last_sync
    {
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

    if activities.len() > 0 {
        info!("Fetched {} activity files", activities.len());

        match import_file_list(&activities) {
            Ok((inserted, latest)) => {
                let mut new_latest = Local::now();
                if let Some(latest) = latest {
                    new_latest = latest;
                }
                let _ = Device::update_latest_sync(serial, new_latest);
                Ok(inserted)
            }
            Err(e) => Err(e),
        }
    } else {
        Ok(0)
    }
}

fn import_file_list<F>(files: &[F]) -> Result<(u16, Option<DateTime<Local>>), String>
where
    F: AsRef<Path>,
{
    let mut success = 0_u16;

    let mut latest: Option<DateTime<Local>> = None;
    for file in files {
        info!("Importing file {}", file.as_ref().display());
        let res = match load_from_file(file.as_ref()) {
            Ok(mut session) => {
                let found = Session::find_by_id(session.timestamp.timestamp(), false)
                    .map(|opt| opt.is_some())
                    .unwrap_or(false);

                if !found {
                    if let Err(e) = session.insert() {
                        Err(format!("Error persisting session: {}", e))
                    } else {
                        success += 1;
                        latest = if let Some(latest_v) = latest {
                            if session.timestamp.timestamp() > latest_v.timestamp() {
                                Some(session.timestamp)
                            } else {
                                latest
                            }
                        } else {
                            Some(session.timestamp)
                        };
                        Ok("Session imported succesfully".to_string())
                    }
                } else {
                    Err(format!(
                        "Session with date {} already exists",
                        session.format_date()
                    ))
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
            Err(e) => {
                error!("  {}", e);

                show_notification(NotificationDefinition {
                    title: format!("{}", file.as_ref().file_name().unwrap().display()),
                    body: e,
                    kind: NotificationKind::Persistant,
                });
            }
        }
    }

    Ok((success, latest))
}
