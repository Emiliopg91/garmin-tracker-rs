pub mod models;
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use chrono::{Datelike, Local, TimeZone, Timelike};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::log::{error, info};

use crate::{
    garmin::{
        database::{
            dao::{exercise::Exercise, serie::Serie, session::Session},
            DATABASE_INST,
        },
        mtp::MtpClient,
    },
    ui::{
        notifications::{models::NotificationDefinition, show_notification},
        sessions::models::{SessionDetails, SessionListItem, SessionSerie, SessionSeriesUpdate},
    },
};

#[tauri::command]
pub fn get_sessions(app: AppHandle) -> Result<Vec<SessionListItem>, String> {
    info!("Getting sessions list...");
    let res: Result<Vec<SessionListItem>, String> = {
        let sessions = Session::load_from_db().map_err(|e| e.to_string())?;

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
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error getting sessions list".to_string(),
                    body: e.clone(),
                },
            );
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_session_details(app: AppHandle, timestamp: i64) -> Result<SessionDetails, String> {
    info!(
        "Getting details for session {}",
        Local.timestamp_opt(timestamp, 0).unwrap().to_rfc3339()
    );
    let res = {
        if let Some(session) = Session::find_by_id(timestamp).map_err(|e| e.to_string())? {
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
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error getting session details".to_string(),
                    body: e.clone(),
                },
            );
            Err(e)
        }
    }
}

#[tauri::command]
pub fn save_session_changes(app: AppHandle, details: SessionSeriesUpdate) -> Result<(), String> {
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
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Session updated succesfully".to_string(),
                    body: "".to_string(),
                },
            );

            Ok(l)
        }
        Err(e) => {
            error!("Error updating session: {}", e);
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error updating session".to_string(),
                    body: e.clone(),
                },
            );
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn import_from_file(app: AppHandle) -> Result<(), String> {
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
            Ok(-1)
        }
    };

    match res {
        Ok(l) => {
            if l >= 0 {
                info!("Imported {} sessions from disk", l);
                let _ = show_notification(
                    app,
                    NotificationDefinition {
                        title: "Import succesful".to_string(),
                        body: format!("Imported {} sessions from disk", l),
                    },
                );
            }

            Ok(())
        }
        Err(e) => {
            error!("Error on sessions import: {}", e);
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error on sessions import".to_string(),
                    body: e.to_string(),
                },
            );
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn import_from_device(app: AppHandle, serial: &str) -> Result<(), String> {
    info!("Starting import from device with S/N {}", serial);
    let latest = Session::find_latest().map_err(|e| e.to_string())?;
    let mut latest_date = "2026-06-08-00-00-00".to_string();
    if let Some(latest) = latest {
        latest_date = format!(
            "{:04}-{:02}-{:02}-{:02}-{:02}-{:02}",
            latest.timestamp.year(),
            latest.timestamp.month(),
            latest.timestamp.day(),
            latest.timestamp.hour(),
            latest.timestamp.minute(),
            latest.timestamp.second(),
        );
    }

    info!(
        "Fetching from device activity files after {}...",
        latest_date
    );
    let activities = MtpClient::download_activities_since(serial, latest_date)
        .await
        .map_err(|e| e.to_string())?;
    info!("Fetched {} activity files", activities.len());

    match import_file_list(&activities) {
        Ok(l) => {
            info!("Imported {} sessions from disk", l);
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Import succesful".to_string(),
                    body: format!("Imported {} sessions from disk", l),
                },
            );

            Ok(())
        }
        Err(e) => {
            error!("Error on sessions import: {}", e);
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error on sessions import".to_string(),
                    body: e.to_string(),
                },
            );
            Err(e)
        }
    }
}

fn import_file_list<F>(files: &[F]) -> Result<isize, String>
where
    F: AsRef<Path>,
{
    let mut res = Ok(-1);

    let mut cur_file = None;
    match DATABASE_INST.lock() {
        Ok(mut db) => {
            match db.run_in_transaction(|tx| {
                let mut inner_res = Ok(());
                for file in files {
                    info!("Importing file {}", file.as_ref().display());
                    cur_file = Some(file);
                    match Session::load_from_file(file.as_ref()) {
                        Ok(mut session) => {
                            session.insert(tx)?;
                        }
                        Err(e) => {
                            inner_res = Err(format!("Error parsing session: {}", e));
                            break;
                        }
                    }
                }
                match inner_res {
                    Ok(_) => {
                        res = Ok(files.len() as isize);
                    }
                    Err(e) => res = Err(e),
                }
                Ok(())
            }) {
                Ok(_) => (),
                Err(e) => {
                    if e.to_string().contains("UNIQUE constraint") {
                        res = Err("Session already imported".to_string());
                    } else {
                        res = Err(format!("Error writing to database: {}", e));
                    }
                }
            }
        }
        Err(e) => {
            res = Err(format!("Error connecting to database: {}", e));
        }
    }

    match res {
        Ok(v) => Ok(v),
        Err(e) => match cur_file {
            Some(f) => Err(format!(
                "{}: {}",
                f.as_ref().file_name().unwrap().display(),
                e
            )),
            None => Err(e),
        },
    }
}
