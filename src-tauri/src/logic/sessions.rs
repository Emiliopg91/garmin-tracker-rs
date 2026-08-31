use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    path::Path,
    sync::Mutex,
    time::Duration,
};

use crate::{
    dao::{
        additional_data::{self, AdditionalDataRepository},
        device::{Device, DeviceRepository},
        exercise::{self, Exercise, ExerciseRepository},
        serie::{self, Serie, SerieRepository, entity},
        session::{self, SessionRepository},
    },
    dto::{
        notifications::{NotificationDefinition, NotificationKind},
        sessions::{SessionDetails, SessionListItem, SessionLocation, SessionSeriesUpdate},
    },
    logic::notifications::show_notification,
    mtp::MTP_CLIENT_INST,
    parser::load_from_file,
    utils::{
        constants::SEMICIRCLE_TO_DEGREES,
        translations::{translate, translate_and_replace},
    },
};
use chrono::{Datelike, Local, TimeZone, Timelike, offset::LocalResult};
use curl::easy::Easy;
use garmin_tracker_rs_macros::traced_command;
use indexmap::IndexMap;
use rayon::prelude::*;
use rusqlite_orm::{
    dao::{
        Repository,
        helpers::types::{order_by::OrderBy, value::Value, where_clause::Where},
    },
    database::{Database, errors::DatabaseError},
};
use tauri::{AppHandle, Emitter};
use tauri_plugin_log::log::{error, info, warn};

/// Returns every recorded session, newest first.
#[traced_command]
#[tauri::command]
pub fn get_sessions() -> Result<Vec<SessionListItem>, String> {
    info!("Getting sessions list...");
    let res = Database::run_in_connection(|conn| {
        let sessions = SessionRepository::select()
            .order_by(OrderBy::Desc(session::entity::columns::DATE))
            .fetch_in(conn)?;

        Ok(sessions
            .iter()
            .map(SessionListItem::from)
            .collect::<Vec<_>>())
    });

    match res {
        Ok(l) => {
            info!("Retreived {} sessions", l.len());
            Ok(l)
        }
        Err(DatabaseError::RunningOnConnection(e)) => {
            error!("Error getting sessions list: {}", e);
            show_notification(NotificationDefinition {
                title: translate("error_session_list"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

/// Returns full details for one session (series grouped by exercise, heart rate, GPS, speeds, device).
#[traced_command]
#[tauri::command]
pub fn get_session_details(timestamp: i32) -> Result<SessionDetails, String> {
    let timestamp = timestamp as i64;
    info!(
        "Getting details for session {}",
        Local.timestamp_opt(timestamp, 0).unwrap().to_rfc3339()
    );

    let res = Database::run_in_connection(|conn| {
        let mut session = SessionRepository::select_by_id_in(conn, timestamp)?.unwrap();

        session.fetch_series_relationship_in_conn(conn)?;
        if session.device.is_some() {
            session.fetch_device_obj_relationship_in_conn(conn)?;
        }
        session.fetch_additional_data_relationship_in_conn(conn)?;

        let condition_set: HashSet<(_, _)> = session
            .series
            .iter()
            .map(|r| (r.ex_cat.clone(), r.ex_id))
            .collect();

        let in_conditions = condition_set
            .into_iter()
            .map(|(cat, id)| vec![cat.into(), id.into()])
            .collect::<Vec<Vec<Value>>>();

        let exercises = ExerciseRepository::select()
            .where_(Where::InMultiple(
                vec![
                    exercise::entity::columns::CATEGORY,
                    exercise::entity::columns::ID,
                ],
                in_conditions,
            ))
            .fetch_in(conn)?;

        let exercise_by_key: HashMap<(_, _), &Exercise> = exercises
            .iter()
            .map(|e| ((e.category.clone(), e.id), e))
            .collect();

        let mut res: IndexMap<Exercise, Vec<Serie>> = IndexMap::with_capacity(exercises.len());
        for r in &session.series {
            if let Some(&ex) = exercise_by_key.get(&(r.ex_cat.clone(), r.ex_id)) {
                res.entry(ex.clone()).or_default().push(r.clone());
            }
        }

        Ok(SessionDetails::from((&session, &res)))
    });

    match res {
        Ok(details) => {
            info!(
                "Found details for session {} @ {}",
                details.sport, details.timestamp
            );
            Ok(details)
        }
        Err(DatabaseError::RunningOnConnection(e)) => {
            error!("Error getting session details: {}", e);
            show_notification(NotificationDefinition {
                title: translate("error_session_details"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

/// Applies user edits (reps/weight) to a session's series and recomputes personal records.
#[traced_command]
#[tauri::command]
pub fn save_session_changes(details: SessionSeriesUpdate) -> Result<(), String> {
    info!(
        "Saving changes on session {}...",
        Local
            .timestamp_opt(details.timestamp as i64, 0)
            .unwrap()
            .to_rfc3339()
    );
    let res = Database::run_in_transaction(|tx| {
        let mut exercises = HashSet::new();
        for serie in &details.series {
            SerieRepository::update()
                .set(entity::columns::REPS, serie.reps.into())
                .set(entity::columns::WEIGHT, serie.weight.into())
                .where_(Where::And(vec![
                    Where::Eq(entity::columns::SESSION, details.timestamp.into()),
                    Where::Eq(entity::columns::IDX, serie.idx.into()),
                ]))
                .execute_in(tx)?;
            exercises.insert((serie.ex_cat.clone(), serie.ex_id));
        }

        let mut session =
            SessionRepository::select_by_id_in(tx, details.timestamp as i64)?.unwrap();
        session.fetch_series_relationship_in_conn(tx)?;
        session.update_by_id_in(tx)?;

        update_prs(tx, exercises, &[details.timestamp as i64])?;
        Ok(())
    });

    match res {
        Ok(l) => {
            info!("Session updated succesfully");
            show_notification(NotificationDefinition {
                title: translate("ok_update_session"),
                body: "".to_string(),
                kind: NotificationKind::Temporal,
            });

            Ok(l)
        }
        Err(DatabaseError::Transaction(e)) => {
            error!("Error updating session: {}", e);
            show_notification(NotificationDefinition {
                title: translate("error_update_session"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

/// Tauri command wrapper around `_import_from_device`; returns the number of sessions imported.
#[traced_command]
#[tauri::command]
pub async fn import_from_device(app: AppHandle, serial: &str) -> Result<usize, String> {
    _import_from_device(&app, serial).await
}

/// Downloads new activity files from the given device since its last sync and imports them.
pub async fn _import_from_device(app: &AppHandle, serial: &str) -> Result<usize, String> {
    info!("Starting import from device with S/N {}", serial);
    let mut latest_date = "2026-06-08-00-00-00".to_string();
    let mut device = DeviceRepository::select_by_id(serial)
        .map_err(|e| e.to_string())?
        .unwrap();

    if let Some(latest) = device.last_sync {
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

    let res = tokio::task::spawn_blocking(move || {
        Database::run_in_transaction(|tx| {
            let res = if !activities.is_empty() {
                info!("Fetched {} activity files", activities.len());
                import_file_list(tx, &activities, &device)
            } else {
                Ok(Vec::new())
            }?;

            device.last_sync = Some(Local::now().timestamp());
            device.update_by_id_in(tx)?;

            Ok(res)
        })
    })
    .await
    .expect("blocking DB task panicked");

    match res {
        Ok(res) => {
            if !res.is_empty() {
                let app = app.clone();
                std::thread::spawn(move || {
                    update_pending_geolocation(&app);
                });
            }
            Ok(res.len())
        }
        Err(DatabaseError::Transaction(e)) => Err(e.deref().to_string()),
        _ => unreachable!(),
    }
}

/// Parses a batch of `.FIT` files in parallel and inserts each new session (plus its exercises/series/heart rate/GPS/speeds) in the given transaction, then refreshes personal records.
fn import_file_list<F>(
    tx: &mut rusqlite_orm::rusqlite::Transaction,
    files: &[F],
    device: &Device,
) -> Result<Vec<i64>, DatabaseError>
where
    F: AsRef<Path> + Sync,
{
    let mut success = Vec::new();
    let mut handled_exercises = HashSet::new();

    let mut sessions = files
        .par_iter()
        .filter_map(|file| {
            info!("Parsing file {}", file.as_ref().display());
            match load_from_file(file.as_ref()) {
                Ok(session) => Some(session),
                Err(e) => {
                    error!("Error parsing session: {}", e);

                    show_notification(NotificationDefinition {
                        title: format!("{}", file.as_ref().file_name().unwrap().display()),
                        body: translate_and_replace("error_parsing_session", &[&e.to_string()]),
                        kind: NotificationKind::Persistant,
                    });

                    None
                }
            }
        })
        .collect::<Vec<_>>();

    sessions.sort_by_key(|s| s.date);

    for mut session in sessions {
        let formatted_time = match Local.timestamp_opt(session.date, 0) {
            LocalResult::Single(fecha) => fecha.format("%H:%M:%S %d/%m/%Y").to_string(),
            _ => "".to_string(),
        };

        let res: Result<bool, String> = {
            info!("Importing session {} - {}", session.workout, formatted_time);
            let added = if SessionRepository::exists_in(tx, session.date)? {
                let msg = format!("Session with date {} already exists", session.date);
                warn!("{}", msg);
                false
            } else {
                session.device = Some(device.serial.to_string());
                SessionRepository::insert()
                    .item(session.clone())
                    .execute_in(tx)?;

                let mut insert = ExerciseRepository::insert().or_ignore();
                let mut seen = HashSet::new();
                for serie in &session.series {
                    let exercise = serie.exercise.clone().unwrap();
                    if seen.insert(exercise.clone()) {
                        insert = insert.item(exercise.clone());
                    }
                    handled_exercises.insert((exercise.category, exercise.id));
                }
                if !seen.is_empty() {
                    insert.execute_in(tx)?;
                }

                let mut insert = SerieRepository::insert();
                let mut count = 0;
                for serie in &session.series {
                    insert = insert.item(serie.clone());
                    count += 1;
                }
                if count > 0 {
                    insert.execute_in(tx)?;
                }

                if let Some(additional_data) = session.additional_data {
                    AdditionalDataRepository::insert()
                        .item(additional_data.clone())
                        .execute_in(tx)?;
                }

                success.push(session.date);

                true
            };

            Ok(added)
        };

        if let Err(e) = res {
            error!("  {}", e);

            show_notification(NotificationDefinition {
                title: format!(
                    "{} | {} | {}",
                    session.sport, session.workout, formatted_time
                ),
                body: e,
                kind: NotificationKind::Persistant,
            });
        }
    }

    if !success.is_empty() {
        show_notification(NotificationDefinition {
            title: translate("ok_import_sessions"),
            body: translate_and_replace("imported_n_sessions", &[&success.len().to_string()]),
            kind: NotificationKind::Temporal,
        });
        update_prs(tx, handled_exercises, &success)?;
    }

    Ok(success)
}

/// Recomputes the `pr` flag for each affected exercise and notifies if any of the just-imported/edited sessions set a new record.
fn update_prs(
    tx: &rusqlite_orm::rusqlite::Transaction,
    exercises: HashSet<(String, u16)>,
    sessions: &[i64],
) -> rusqlite_orm::database::errors::Result<()> {
    let mut new_prs = false;

    let sessions = sessions.to_vec();

    let mut update_false_conditions = vec![];
    let mut update_true_conditions = vec![];
    for exer in &exercises {
        update_false_conditions.push(vec![exer.0.clone().into(), exer.1.into()]);
        if let Some(pr) = SerieRepository::select()
            .where_(Where::And(vec![
                Where::Eq(serie::entity::columns::EX_CAT, exer.0.clone().into()),
                Where::Eq(serie::entity::columns::EX_ID, exer.1.into()),
            ]))
            .order_by(OrderBy::Desc(entity::columns::WEIGHT))
            .order_by(OrderBy::Desc(entity::columns::REPS))
            .order_by(OrderBy::Asc(entity::columns::SESSION))
            .order_by(OrderBy::Asc(entity::columns::IDX))
            .limit(1)
            .fetch_one_in(tx)?
        {
            update_true_conditions.push(vec![pr.session.into(), pr.idx.into()]);
            new_prs = new_prs || sessions.contains(&pr.session);
        }
    }

    if !update_true_conditions.is_empty() {
        SerieRepository::update()
            .set(serie::entity::columns::PR, false.into())
            .where_(Where::And(vec![
                Where::InMultiple(
                    vec![
                        serie::entity::columns::EX_CAT,
                        serie::entity::columns::EX_ID,
                    ],
                    update_false_conditions,
                ),
                Where::Eq(serie::entity::columns::PR, true.into()),
            ]))
            .execute_in(tx)?;
        SerieRepository::update()
            .set(serie::entity::columns::PR, true.into())
            .where_(Where::InMultiple(
                vec![serie::entity::columns::SESSION, serie::entity::columns::IDX],
                update_true_conditions,
            ))
            .execute_in(tx)?;
    }

    if new_prs {
        show_notification(NotificationDefinition {
            title: translate("new_record"),
            body: translate("contratulations_pr"),
            kind: NotificationKind::Temporal,
        });
    }

    Ok(())
}

static GELOCATION_MUTEX: Mutex<bool> = Mutex::new(false);

/// Recover all pending workouts pending on geolocation
pub fn update_pending_geolocation(app: &AppHandle) {
    let _lock = GELOCATION_MUTEX.lock().unwrap();
    info!("Looking for pending geocode workouts...");
    match Database::run_in_connection(|conn| {
        let unnamed_sessions = SessionRepository::select()
            .where_(Where::Eq(session::entity::columns::WORKOUT, "".into()))
            .fetch_in(conn)?;

        let sessions_ids = unnamed_sessions
            .iter()
            .map(|s| s.date.into())
            .collect::<Vec<_>>();

        let pending = AdditionalDataRepository::select()
            .where_(Where::And(vec![
                Where::In(additional_data::entity::columns::SESSION, sessions_ids),
                Where::NotNull(additional_data::entity::columns::COORDINATES),
            ]))
            .order_by(OrderBy::Desc(additional_data::entity::columns::SESSION))
            .fetch_in(conn)?;

        Ok(pending)
    }) {
        Ok(pendings) => {
            info!("Found {} pending sessions", pendings.len());
            for pending in pendings {
                let first = pending
                    .get_coordinates_semicircle()
                    .unwrap()
                    .iter()
                    .find(|e| e.is_some())
                    .unwrap()
                    .unwrap();

                match get(
                    first.0 as f64 * SEMICIRCLE_TO_DEGREES,
                    first.1 as f64 * SEMICIRCLE_TO_DEGREES,
                ) {
                    Ok(response) => match serde_json::from_str::<serde_json::Value>(&response) {
                        Ok(e) => {
                            if let Some(address) = e.get("address") {
                                for key in ["village", "town", "city", "state", "country"] {
                                    if let Some(value) = address.get(key) {
                                        let location = value.to_string().replace("\"", "");

                                        match SessionRepository::update()
                                            .set(
                                                session::entity::columns::WORKOUT,
                                                location.clone().into(),
                                            )
                                            .where_(Where::Eq(
                                                session::entity::columns::DATE,
                                                pending.session.into(),
                                            ))
                                            .execute()
                                        {
                                            Ok(1) => {
                                                info!(
                                                    "Updated location for session @ {}",
                                                    pending.session
                                                );
                                                let payload: SessionLocation = SessionLocation {
                                                    session: pending.session as i32,
                                                    location,
                                                };
                                                let _ =
                                                    app.emit("session_location_update", payload);
                                            }
                                            Ok(_) => {
                                                info!("Missing session @ {}", pending.session)
                                            }
                                            Err(e) => {
                                                error!(
                                                    "Error while updating session {}: {}",
                                                    pending.session, e
                                                )
                                            }
                                        }

                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error parsing response: {}", e)
                        }
                    },
                    Err(e) => {
                        error!("Error on geocode query: {}", e)
                    }
                }

                std::thread::sleep(Duration::from_secs(1));
            }
        }
        Err(e) => {
            error!("Error while looking for pending sessions: {}", e);
        }
    }
}

/// Performs a blocking HTTP GET and returns the response body as a string.
fn get(lat: f64, lon: f64) -> Result<String, curl::Error> {
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}",
        lat, lon
    );
    let mut easy = Easy::new();
    easy.url(&url)?;
    easy.useragent("garmin-tracker-rs")?;

    let mut data = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|chunk| {
            data.extend_from_slice(chunk);
            Ok(chunk.len())
        })?;
        transfer.perform()?;
    }

    Ok(String::from_utf8_lossy(&data).to_string())
}
