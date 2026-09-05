use std::{collections::HashSet, fs, path::Path, sync::Mutex, time::Duration};

use crate::{
    SettingsLock,
    dao::{
        additional_data::{self, AdditionalDataRepository},
        device::{Device, DeviceRepository},
        exercise::{self, ExerciseRepository},
        serie::{self, SerieRepository, entity},
        session::{self, SessionRepository},
        workout::{Workout, WorkoutRepository},
    },
    dto::{
        notifications::{NotificationDefinition, NotificationKind},
        sessions::{SessionDetails, SessionListItem, SessionLocation, SessionSeriesUpdate},
    },
    logic::{notifications::show_notification, report_error},
    mtp::MTP_CLIENT_INST,
    parser::{FitParser, errors::ParseFitFileError},
    utils::translations::{Languages, translate, translate_and_replace},
};
use chrono::{Datelike, Local, TimeZone, Timelike, offset::LocalResult};
use curl_rest::StatusCode;
use garmin_tracker_rs_macros::traced_command;
use rayon::prelude::*;
use rusqlite_orm::{
    dao::Repository,
    database::DatabasePool,
    errors::DatabaseError,
    types::{order_by::OrderBy, value::Value, where_clause::Where},
};
use rustyfit::Decoder;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_log::log::{error, info, warn};

/// Returns every recorded session, newest first.
#[traced_command]
#[tauri::command]
pub fn get_sessions(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
) -> Result<Vec<SessionListItem>, String> {
    info!("Getting sessions list...");
    let res = database.run_in_connection(|conn| {
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
        Err(e) => Err(report_error(
            e,
            settings.read().unwrap().language,
            "error_session_list",
            "Error getting sessions list",
        )),
    }
}

/// Returns full details for one session (series grouped by exercise, heart rate, GPS, speeds, device).
#[traced_command]
#[tauri::command]
pub fn get_session_details(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
    timestamp: i32,
) -> Result<SessionDetails, String> {
    let timestamp = timestamp as i64;
    info!(
        "Getting details for session {}",
        Local.timestamp_opt(timestamp, 0).unwrap().to_rfc3339()
    );

    let res = database.run_in_connection(|conn| {
        let mut session = SessionRepository::select_by_id_in(conn, timestamp)?.unwrap();

        session.fetch_series_relationship_in_conn(conn)?;
        if session.device.is_some() {
            session.fetch_device_obj_relationship_in_conn(conn)?;
        }
        session.fetch_additional_data_relationship_in_conn(conn)?;

        let condition_set: HashSet<(_, _)> =
            session.series.iter().map(|r| (r.ex_cat, r.ex_id)).collect();

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

        Ok(SessionDetails::from((
            &session,
            exercises.as_slice(),
            session.series.as_slice(),
        )))
    });

    match res {
        Ok(details) => {
            info!("Found details for session {}", details.timestamp);
            Ok(details)
        }
        Err(e) => Err(report_error(
            e,
            settings.read().unwrap().language,
            "error_session_details",
            "Error getting session details",
        )),
    }
}

/// Applies user edits (reps/weight) to a session's series and recomputes personal records.
#[traced_command]
#[tauri::command]
pub fn save_session_changes(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
    details: SessionSeriesUpdate,
) -> Result<(), String> {
    info!(
        "Saving changes on session {}...",
        Local
            .timestamp_opt(details.timestamp as i64, 0)
            .unwrap()
            .to_rfc3339()
    );
    let lang = settings.read().unwrap().language;
    let res = database.run_in_transaction(|tx| {
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
            exercises.insert((serie.ex_cat, serie.ex_id));
        }

        update_prs(tx, exercises, &[details.timestamp as i64], lang)?;
        Ok(())
    });

    match res {
        Ok(l) => {
            info!("Session updated succesfully");
            show_notification(NotificationDefinition {
                title: translate("ok_update_session", lang),
                body: "".to_string(),
                kind: NotificationKind::Temporal,
            });

            Ok(l)
        }
        Err(e) => Err(report_error(
            e,
            lang,
            "error_update_session",
            "Error updating session",
        )),
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
    let lang = app.state::<SettingsLock>().read().unwrap().language;
    let db = app.state::<DatabasePool>();
    let mut device = DeviceRepository::select_by_id(&db, serial)
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
    let mut res = Ok(Vec::new());
    let mut activities = Vec::new();
    let mut src_dir = None;

    if let Ok(Some(dst_dir)) = MTP_CLIENT_INST
        .lock()
        .await
        .download_activities_since(serial, latest_date)
        .await
        .map_err(|e| e.to_string())
    {
        src_dir = Some(dst_dir.clone());
        activities = Vec::new();

        if let Ok(read_dir) = fs::read_dir(dst_dir) {
            for entry in read_dir {
                if let Ok(entry) = entry
                    && entry.file_type().unwrap().is_file()
                {
                    activities.push(entry.path());
                }
            }
        };

        let activities_cpy = activities.clone();
        let app_cpy = app.clone();
        res = tokio::task::spawn_blocking(move || {
            let db = app_cpy.state::<DatabasePool>();
            db.run_in_transaction(|tx| {
                let res = if !activities_cpy.is_empty() {
                    info!("Fetched {} activity files", activities_cpy.len());
                    import_file_list(tx, &activities_cpy, &device, lang)
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
    }

    match res {
        Ok(res) => {
            if res.len() == activities.len()
                && let Some(src_dir) = src_dir
            {
                let _ = fs::remove_dir_all(src_dir);
            }

            if !res.is_empty() {
                let app = app.clone();
                std::thread::spawn(move || {
                    let db = app.state::<DatabasePool>();
                    update_pending_geolocation(&app, &db);
                });
            }
            Ok(res.len())
        }
        Err(e) => Err(report_error(
            e,
            lang,
            "error_import_sessions",
            "Error importing sessions",
        )),
    }
}

/// Parses a batch of `.FIT` files in parallel and inserts each new session (plus its exercises/series/heart rate/GPS/speeds) in the given transaction, then refreshes personal records.
fn import_file_list<F>(
    tx: &mut rusqlite_orm::rusqlite::Transaction,
    files: &[F],
    device: &Device,
    lang: Languages,
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
            let res = match FitParser::from_file(file, &mut Decoder::new()) {
                Ok(parser) => match parser.parse_session() {
                    Ok(session) => Ok((session, file)),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            };
            match res {
                Ok(result) => Some(result),
                Err(e) => {
                    error!("Error parsing session: {}", e);

                    let error_msg = match &e {
                        ParseFitFileError::UnknownExercise(category, id) => translate_and_replace(
                            "error_parser_unknown_exercise",
                            &[&category.to_string(), &id.to_string()],
                            lang,
                        ),
                        other => other.to_string(),
                    };

                    show_notification(NotificationDefinition {
                        title: format!("{}", file.as_ref().file_name().unwrap().display()),
                        body: translate_and_replace("error_parsing_session", &[&error_msg], lang),
                        kind: NotificationKind::Persistant,
                    });

                    None
                }
            }
        })
        .collect::<Vec<_>>();

    sessions.sort_by_key(|s| s.0.date);

    for (mut session, file) in sessions {
        let formatted_time = match Local.timestamp_opt(session.date, 0) {
            LocalResult::Single(fecha) => fecha.format("%H:%M:%S %d/%m/%Y").to_string(),
            _ => "".to_string(),
        };

        let res: Result<bool, String> = {
            info!("Importing session {} - {}...", session.name, formatted_time);
            let added = if SessionRepository::exists_in(tx, session.date)? {
                let msg = format!("Session with date {} already exists", session.date);
                warn!("{}", msg);
                false
            } else {
                let date = session.date;
                let series = std::mem::take(&mut session.series);
                let add_data = session.additional_data.take();

                if let Some(workout) = &session.workout {
                    WorkoutRepository::insert()
                        .or_ignore()
                        .item(Workout {
                            name: workout.to_string(),
                        })
                        .execute_in(tx)?;
                }

                session.device = Some(device.serial.to_string());
                SessionRepository::insert().item(session).execute_in(tx)?;

                for serie in &series {
                    handled_exercises.insert((serie.ex_cat, serie.ex_id));
                }

                let mut insert = SerieRepository::insert();
                let mut count = 0;
                for serie in series {
                    insert = insert.item(serie);
                    count += 1;
                }
                if count > 0 {
                    insert.execute_in(tx)?;
                }

                if let Some(additional_data) = add_data {
                    AdditionalDataRepository::insert()
                        .item(additional_data)
                        .execute_in(tx)?;
                }

                success.push(date);

                let _ = fs::remove_file(file);
                #[cfg(debug_assertions)]
                {
                    use std::path::PathBuf;

                    let file = PathBuf::from(format!("{}.txt", file.as_ref().display()));
                    let _ = fs::remove_file(file);
                }

                true
            };

            Ok(added)
        };

        if let Err(e) = res {
            error!("  {}", e);

            show_notification(NotificationDefinition {
                title: formatted_time,
                body: e,
                kind: NotificationKind::Persistant,
            });
        }
    }

    if !success.is_empty() {
        show_notification(NotificationDefinition {
            title: translate("ok_import_sessions", lang),
            body: translate_and_replace("imported_n_sessions", &[&success.len().to_string()], lang),
            kind: NotificationKind::Temporal,
        });
        update_prs(tx, handled_exercises, &success, lang)?;
    }

    Ok(success)
}

/// Recomputes the `pr` flag for each affected exercise and notifies if any of the just-imported/edited sessions set a new record.
fn update_prs(
    tx: &rusqlite_orm::rusqlite::Transaction,
    exercises: HashSet<(u16, u16)>,
    sessions: &[i64],
    lang: Languages,
) -> rusqlite_orm::errors::Result<()> {
    let mut new_prs = false;

    let sessions = sessions.to_vec();

    let mut update_false_conditions = vec![];
    let mut update_true_conditions = vec![];

    for exer in &exercises {
        update_false_conditions.push(vec![exer.0.into(), exer.1.into()]);
        if let Some(pr) = SerieRepository::select()
            .where_(Where::And(vec![
                Where::Eq(serie::entity::columns::EX_CAT, exer.0.into()),
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
            title: translate("new_record", lang),
            body: translate("contratulations_pr", lang),
            kind: NotificationKind::Temporal,
        });
    }

    Ok(())
}

static GELOCATION_MUTEX: Mutex<bool> = Mutex::new(false);

/// Recover all pending workouts pending on geolocation
pub fn update_pending_geolocation(app: &AppHandle, db: &DatabasePool) {
    let _lock = GELOCATION_MUTEX.lock().unwrap();
    info!("Looking for pending geocode workouts...");
    match db.run_in_connection(|conn| {
        let unnamed_sessions = SessionRepository::select()
            .where_(Where::Eq(session::entity::columns::NAME, "".into()))
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
                    .get_coordinates_degrees()
                    .unwrap()
                    .iter()
                    .find(|e| e.is_some())
                    .unwrap()
                    .unwrap();

                let url = format!(
                    "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}",
                    first.0, first.1
                );
                //
                let resp = curl_rest::Client::with_user_agent("garmin-tracker-rs")
                    .get()
                    .header(curl_rest::Header::Accept("application/json".into()))
                    .send(&url);

                match resp {
                    Ok(response) => {
                        if response.status == StatusCode::Ok {
                            match serde_json::from_slice::<serde_json::Value>(&response.body) {
                                Ok(e) => {
                                    if let Some(address) = e.get("address") {
                                        for key in ["village", "town", "city", "state", "country"] {
                                            if let Some(value) = address.get(key) {
                                                let location = value.to_string().replace("\"", "");

                                                match SessionRepository::update()
                                                    .set(
                                                        session::entity::columns::NAME,
                                                        location.clone().into(),
                                                    )
                                                    .where_(Where::Eq(
                                                        session::entity::columns::DATE,
                                                        pending.session.into(),
                                                    ))
                                                    .execute(db)
                                                {
                                                    Ok(1) => {
                                                        info!(
                                                            "Updated location for session @ {}",
                                                            pending.session
                                                        );
                                                        let payload: SessionLocation =
                                                            SessionLocation {
                                                                session: pending.session as i32,
                                                                location,
                                                            };
                                                        let _ = app.emit(
                                                            "session_location_update",
                                                            payload,
                                                        );
                                                    }
                                                    Ok(_) => {
                                                        info!(
                                                            "Missing session @ {}",
                                                            pending.session
                                                        )
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
                            }
                        } else {
                            error!("Invalid status code: {}", response.status)
                        }
                    }
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
