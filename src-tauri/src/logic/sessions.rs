use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    path::Path,
};

use chrono::{Datelike, Local, TimeZone, Timelike};
use garmin_tracker_rs_macros::{traced_command, translate};
use indexmap::IndexMap;
use rusqlite_orm::{
    dao::{
        Repository,
        helpers::types::{order_by::OrderBy, value::Value, where_clause::Where},
    },
    database::{Database, errors::DatabaseError},
};
use tauri_plugin_log::log::{error, info, warn};

use crate::{
    dao::{
        device::{Device, DeviceRepository},
        exercise::{self, Exercise, ExerciseRepository},
        gps_coordinates::GpsCoordinatesRepository,
        heart_rate::HeartRateRepository,
        serie::{self, Serie, SerieRepository, entity},
        session::{self, SessionRepository},
    },
    dto::{
        notifications::{NotificationDefinition, NotificationKind},
        sessions::{SessionDetails, SessionListItem, SessionSeriesUpdate},
    },
    logic::notifications::show_notification,
    mtp::MTP_CLIENT_INST,
    parser::load_from_file,
    utils::date_time_utils::DateTimeUtils,
};

#[traced_command]
#[tauri::command]
pub fn get_sessions() -> Result<Vec<SessionListItem>, String> {
    info!("Getting sessions list...");
    let res = Database::run_in_connection(|conn| {
        let sessions = SessionRepository::select()
            .order_by(OrderBy::Desc(session::entity::columns::DATE))
            .fetch_in(conn)?;

        Ok(sessions
            .into_iter()
            .map(|s| SessionListItem::from(&s))
            .collect::<Vec<_>>())
    });

    match res {
        Ok(l) => {
            info!("Retreived {} sessions", l.len());
            Ok(l)
        }
        Err(DatabaseError::Transaction(e)) => {
            error!("Error getting sessions list: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_session_list"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

#[traced_command]
#[tauri::command]
pub fn get_session_details(timestamp: i64) -> Result<SessionDetails, String> {
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
        session.fetch_heart_rates_relationship_in_conn(conn)?;
        session.fetch_gps_coordinates_relationship_in_conn(conn)?;

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
                "Found details for session {} - {}",
                details.name, details.date
            );
            Ok(details)
        }
        Err(DatabaseError::Transaction(e)) => {
            error!("Error getting session details: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_session_details"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
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

        let mut session = SessionRepository::select_by_id_in(tx, details.timestamp)?.unwrap();
        session.fetch_series_relationship_in_conn(tx)?;

        session.volume = 0_f64;
        for ser in &session.series {
            session.volume += ser.weight * (ser.reps as f64)
        }
        session.update_by_id_in(tx)?;

        update_prs(tx, exercises)?;
        Ok(())
    });

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
        Err(DatabaseError::Transaction(e)) => {
            error!("Error updating session: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_update_session"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

#[traced_command]
#[tauri::command]
pub async fn import_from_device(serial: &str) -> Result<u16, String> {
    _import_from_device(serial).await
}

pub async fn _import_from_device(serial: &str) -> Result<u16, String> {
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
                Ok(0_u16)
            }?;

            device.last_sync = Some(Local::now().timestamp());
            device.update_by_id_in(tx)?;

            Ok(res)
        })
    })
    .await
    .expect("blocking DB task panicked");

    match res {
        Ok(res) => Ok(res),
        Err(DatabaseError::Transaction(e)) => Err(e.deref().to_string()),
        _ => unreachable!(),
    }
}

fn import_file_list<F>(
    tx: &mut rusqlite_orm::rusqlite::Transaction,
    files: &[F],
    device: &Device,
) -> Result<u16, DatabaseError>
where
    F: AsRef<Path>,
{
    let mut success = 0_u16;
    let mut handled_exercises = HashSet::new();

    let mut latest: Option<i64> = None;
    for file in files {
        info!("Importing file {}", file.as_ref().display());
        let res = match load_from_file(file.as_ref()) {
            Ok(mut session) => {
                let added = if SessionRepository::exists_in(tx, session.date)? {
                    let msg = format!(
                        "Session with date {} already exists",
                        DateTimeUtils::format_time_date(session.date)
                    );
                    warn!("{}", msg);
                    false
                } else {
                    let sp = tx.savepoint().map_err(DatabaseError::Savepoint)?;

                    session.device = Some(device.serial.to_string());
                    SessionRepository::insert()
                        .item(session.clone())
                        .execute_in(&sp)?;

                    let mut insert = ExerciseRepository::insert().or_ignore(true);
                    let mut seen = HashSet::new();
                    for serie in &session.series {
                        let exercise = serie.exercise.clone().unwrap();
                        if seen.insert(exercise.clone()) {
                            insert = insert.item(exercise.clone());
                        }
                        handled_exercises.insert((exercise.category, exercise.id));
                    }
                    if !seen.is_empty() {
                        insert.execute_in(&sp)?;
                    }

                    let mut insert = SerieRepository::insert();
                    let mut count = 0;
                    for serie in &session.series {
                        insert = insert.item(serie.clone());
                        count += 1;
                    }
                    if count > 0 {
                        insert.execute_in(&sp)?;
                    }

                    if let Some(heart_rates) = session.heart_rates {
                        HeartRateRepository::insert()
                            .item(heart_rates.clone())
                            .execute_in(&sp)?;
                    }

                    if let Some(coordinates) = session.gps_coordinates {
                        GpsCoordinatesRepository::insert()
                            .item(coordinates.clone())
                            .execute_in(&sp)?;
                    }

                    sp.commit().map_err(DatabaseError::Savepoint)?;

                    true
                };

                if added {
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
                }

                Ok(added)
            }
            Err(e) => {
                error!("Error parsing session: {}", e);
                Err(translate!("error_parsing_session", e))
            }
        };

        match res {
            Ok(added) if added => {
                info!("Session imported succesfully");

                show_notification(NotificationDefinition {
                    title: format!("{}", file.as_ref().file_name().unwrap().display()),
                    body: translate!("imported_session"),
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

    if success > 0 {
        update_prs(tx, handled_exercises)?;
    }

    Ok(success)
}

fn update_prs(
    tx: &rusqlite_orm::rusqlite::Transaction,
    exercises: HashSet<(String, u16)>,
) -> rusqlite_orm::database::errors::Result<()> {
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

    Ok(())
}
