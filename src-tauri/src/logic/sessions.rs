use std::{collections::HashSet, path::Path};

use chrono::{Datelike, Local, TimeZone, Timelike};
use garmin_tracker_rs_macros::{traced_command, translate};
use rusqlite_orm::{
    dao::{
        Repository,
        helpers::types::{order_by::OrderBy, where_clause::Where},
    },
    database::DATABASE_INST,
};
use tauri_plugin_log::log::{error, info, warn};

use crate::{
    dao::{
        device::DeviceRepository,
        exercise::{self, ExerciseRepository},
        heart_rate::HeartRateRepository,
        serie::{SerieRepository, entity},
        session::{Session, SessionRepository},
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

    match Session::find_by_id(timestamp, true).map_err(|e| e.to_string()) {
        Ok(Some(l)) => {
            let details = SessionDetails::from(&l);
            info!(
                "Found details for session {} - {}",
                details.name, details.date
            );
            Ok(details)
        }
        Ok(None) => {
            info!("Session not found");
            Err("Session not found".to_string())
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
            let db_serie = SerieRepository::select_by_id(details.timestamp, serie.idx)
                .map_err(|e| e.to_string())?;
            if let Some(mut db_serie) = db_serie {
                db_serie.reps = serie.reps;
                db_serie.weight = serie.weight;
                to_update.push(db_serie);
            }
        }
        let exercises = ExerciseRepository::select()
            .order_by(OrderBy::Asc(exercise::entity::columns::NAME))
            .fetch()
            .map_err(|e| e.to_string())?;

        let mut db = DATABASE_INST.lock().map_err(|e| e.to_string())?;
        db.run_in_tx(move |tx| {
            for to_upd in &to_update {
                SerieRepository::update()
                    .set(entity::columns::REPS, to_upd.reps.into())
                    .set(entity::columns::WEIGHT, to_upd.weight.into())
                    .where_(Where::And(vec![
                        Where::Eq(entity::columns::SESSION, to_upd.session.into()),
                        Where::Eq(entity::columns::IDX, to_upd.idx.into()),
                    ]))
                    .execute_in_tx(tx)?;
            }
            for exer in &exercises {
                if let Ok(new_prs) = SerieRepository::select_by_ex_cat_and_ex_id_in_tx(
                    tx,
                    &exer.category,
                    exer.id,
                    Some(&[
                        OrderBy::Desc(entity::columns::WEIGHT),
                        OrderBy::Desc(entity::columns::REPS),
                        OrderBy::Asc(entity::columns::SESSION),
                    ]),
                ) {
                    let _ = SerieRepository::update()
                        .set(entity::columns::PR, false.into())
                        .where_(Where::And(vec![
                            Where::Eq(entity::columns::EX_CAT, exer.category.clone().into()),
                            Where::Eq(entity::columns::EX_ID, exer.id.into()),
                        ]))
                        .execute_in_tx(tx);

                    for mut pr in new_prs {
                        pr.pr = true;
                        let _ = pr.update_by_id_in_tx(tx);
                    }
                }
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
pub async fn import_from_device(serial: &str) -> Result<u16, String> {
    _import_from_device(serial).await
}

pub async fn _import_from_device(serial: &str) -> Result<u16, String> {
    info!("Starting import from device with S/N {}", serial);
    let mut latest_date = "2026-06-08-00-00-00".to_string();
    let mut device = None;

    if let Ok(dev) = DeviceRepository::select_by_id(serial)
        && let Some(dev) = dev
    {
        device = Some(dev.clone());
        if let Some(latest) = dev.last_sync {
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

    let res = if !activities.is_empty() {
        info!("Fetched {} activity files", activities.len());
        import_file_list(&activities)
    } else {
        Ok(0)
    };

    if let Some(mut dev) = device {
        dev.last_sync = Some(Local::now().timestamp());
        let _ = dev.update_by_id();
    }

    res
}

fn import_file_list<F>(files: &[F]) -> Result<u16, String>
where
    F: AsRef<Path>,
{
    let mut success = 0_u16;

    let mut latest: Option<i64> = None;
    for file in files {
        info!("Importing file {}", file.as_ref().display());
        let res = match load_from_file(file.as_ref()) {
            Ok(session) => {
                let mut db = DATABASE_INST.lock().unwrap();
                let imp_res = db.run_in_tx(|tx| {
                    if SessionRepository::exists_in_tx(tx, session.date)? {
                        let msg = format!(
                            "Session with date {} already exists",
                            DateTimeUtils::format_time_date(session.date)
                        );
                        warn!("{}", msg);
                        Ok(false)
                    } else {
                        SessionRepository::insert()
                            .item(session.clone())
                            .execute_in_tx(tx)?;

                        let mut insert = ExerciseRepository::insert().or_ignore(true);
                        let mut seen = HashSet::new();
                        for exercise in session.series.iter().map(|e| e.0) {
                            if seen.insert(exercise.clone()) {
                                insert = insert.item(exercise.clone());
                            }
                        }
                        if !seen.is_empty() {
                            insert.execute_in_tx(tx)?;
                        }

                        let mut insert = SerieRepository::insert();
                        let mut count = 0;
                        for series in session.series.iter().map(|e| e.1) {
                            for serie in series {
                                insert = insert.item(serie.clone());
                                count += 1;
                            }
                        }
                        if count > 0 {
                            insert.execute_in_tx(tx)?;
                        }

                        let mut insert = HeartRateRepository::insert();
                        for hr in &session.heart_rates {
                            insert = insert.item(hr.clone());
                        }
                        insert.execute_in_tx(tx)?;

                        Ok(true)
                    }
                });
                match imp_res {
                    Ok(added) => {
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
                        error!("Error persisting session: {}", e);
                        Err(translate!("error_persisting_session", e))
                    }
                }
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

    Ok(success)
}
