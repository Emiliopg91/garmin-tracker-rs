use std::collections::HashMap;

use garmin_tracker_rs_macros::traced_command;
use rusqlite_orm::{
    dao::Repository,
    database::DatabasePool,
    types::{order_by::OrderBy, value::Value, where_clause::Where},
};
use tauri::{AppHandle, State};
use tauri_plugin_log::log::info;
use tokio::fs;

use crate::{
    SettingsLock,
    dao::{
        session::{self, SessionRepository, entity},
        set::{self, SetRepository},
        workout::{self, Workout, WorkoutRepository},
        workout_step::{self, WorkoutStepRepository},
    },
    dto::{
        notifications::{NotificationDefinition, NotificationKind},
        workouts::{WorkoutDetails, WorkoutListItem, WorkoutSession},
    },
    fit::writer::FitWriter,
    logic::{notifications::show_notification, report_error, run_blocking},
    mtp::MTP_CLIENT_INST,
    utils::translations::translate,
};

/// Returns sessions grouped/aggregated by workout name (count, average time, latest date), sorted by name.
#[traced_command]
#[tauri::command]
pub async fn get_workout_list(
    app: AppHandle,
    settings: State<'_, SettingsLock>,
) -> Result<Vec<WorkoutListItem>, String> {
    info!("Getting workouts list...");
    let res = run_blocking(app, move |database| {
        database.run_in_connection(|conn| {
            let workouts = WorkoutRepository::select()
                .fetch_in(conn)?
                .into_iter()
                .collect::<Vec<_>>();

            let sessions = SessionRepository::select()
                .where_(Where::NotNull(session::entity::columns::WORKOUT))
                .order_by(OrderBy::Desc(entity::columns::DATE))
                .fetch_in(conn)?;

            let mut workout_stats = HashMap::new();
            workouts.iter().for_each(|w| {
                workout_stats.insert(w.name.clone(), (0, 0, None));
            });

            let enabled_workouts = workouts
                .iter()
                .filter_map(|w| {
                    if w.enabled {
                        Some(w.name.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            sessions.iter().for_each(|s| {
                let entry = workout_stats
                    .entry(s.name.clone())
                    .or_insert((0_u32, 0_u32, None));
                entry.0 += 1_u32;
                entry.1 += s.total_elapsed_time;
                if entry.2.is_none() {
                    entry.2 = Some(s.date);
                }
            });

            let mut res = workout_stats
                .into_iter()
                .map(|wd| WorkoutListItem {
                    enabled: enabled_workouts.contains(&wd.0),
                    name: wd.0,
                    sessions: wd.1.0,
                    avg_time: wd.1.1.checked_div(wd.1.0).unwrap_or(0),
                    latest_session: wd.1.2,
                })
                .collect::<Vec<_>>();

            res.sort_by(|a, b| a.name.cmp(&b.name));

            Ok(res)
        })
    })
    .await;
    match res {
        Ok(l) => {
            info!("Retreived {} workouts", l.len());
            Ok(l)
        }
        Err(e) => Err(report_error(
            e,
            settings.read().unwrap().language,
            "error_workout_list",
            "Error getting workouts list",
        )),
    }
}

/// Returns every session for one workout name with per-session volume and session-over-session volume diff.
#[traced_command]
#[tauri::command]
pub async fn get_workout_details(
    app: AppHandle,
    settings: State<'_, SettingsLock>,
    name: &str,
) -> Result<WorkoutDetails, String> {
    let name = name.to_string();
    let res = run_blocking(app, move |database| {
        database.run_in_connection(|conn| {
            info!("Getting details for workout {}", name);

            let mut workout = WorkoutRepository::select_by_id_in(conn, &name)?.unwrap();
            workout.fetch_steps_relationship_in(conn)?;
            workout.steps.sort_by_key(|e| e.idx);

            let sessions = SessionRepository::select_by_name_in(
                conn,
                &name,
                Some(&[OrderBy::Desc(entity::columns::DATE)]),
            )?;

            let mut latest = sessions.first().map(|latest| latest.date);
            let mut count = 0_u32;
            let mut time = 0_u32;
            let mut volume = 0_f32;

            let series = SetRepository::select()
                .where_(Where::In(
                    set::entity::columns::SESSION,
                    sessions.iter().map(|s| Value::from(s.date)).collect(),
                ))
                .fetch_in(conn)?;

            let mut volume_by_session = HashMap::new();
            for serie in &series {
                *volume_by_session.entry(serie.session).or_insert(0.0) +=
                    (serie.reps as f32) * serie.weight;
            }

            let mut session_list = Vec::new();
            for session in &sessions {
                if latest.is_none() {
                    latest = Some(session.date);
                } else if let Some(latest_o) = latest
                    && session.date > latest_o
                {
                    latest = Some(session.date);
                }

                count += 1;
                time += session.total_elapsed_time;

                let local_volume = volume_by_session.get(&session.date).copied().unwrap_or(0.0);
                volume += local_volume;
                let mut wk_sess = WorkoutSession::from(session);
                wk_sess.volume = local_volume;
                session_list.push(wk_sess);
            }

            let details = WorkoutDetails {
                name: name.to_string(),
                avg_time: if !sessions.is_empty() {
                    time / (sessions.len() as u32)
                } else {
                    0
                },
                latest_session: latest,
                avg_volume: volume / (sessions.len() as f32),
                session_count: count,
                sessions: session_list,
                enabled: workout.enabled,
                steps: workout.steps,
            };

            Ok(details)
        })
    })
    .await;

    match res {
        Ok(l) => {
            info!("Found details for workout {}", l.name);
            Ok(l)
        }
        Err(e) => Err(report_error(
            e,
            settings.read().unwrap().language,
            "error_workout_details",
            "Error getting workout details",
        )),
    }
}

#[traced_command]
#[tauri::command]
pub async fn set_workout_status(app: AppHandle, workout: &str, status: bool) -> Result<(), String> {
    let workout = workout.to_string();
    run_blocking(app, move |database| {
        WorkoutRepository::update()
            .where_(Where::Eq(workout::entity::columns::NAME, workout.into()))
            .set(workout::entity::columns::ENABLED, status.into())
            .execute(database)
            .map(|_| ())
            .map_err(|e| e.to_string())
    })
    .await
}

#[traced_command]
#[tauri::command]
pub async fn send_to_device(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
    workout: &str,
    serial: &str,
) -> Result<(), String> {
    let (lang, weight_unit) = {
        let settings = settings.read().unwrap();
        (settings.language, settings.weight_unit)
    };

    let res: Result<(), String> = async {
        let workout = database
            .run_in_connection(|conn| {
                let mut wk = WorkoutRepository::select_by_id_in(conn, workout)?.unwrap();
                wk.fetch_steps_relationship_in(conn)?;

                Ok(wk)
            })
            .map_err(|e| e.to_string())?;

        let path = std::env::temp_dir().join(workout.get_workout_file_name());
        FitWriter::from(&workout)
            .write(path.clone(), lang, weight_unit)
            .map_err(|e| e.to_string())?;

        let upload = MTP_CLIENT_INST
            .lock()
            .await
            .upload_workout(serial, path.clone())
            .await
            .map_err(|e| e.to_string());

        let _ = fs::remove_file(path).await;

        upload
    }
    .await;

    match res {
        Ok(_) => {
            info!("Workout '{}' sent to device {}", workout, serial);
            show_notification(NotificationDefinition {
                title: workout.to_string(),
                body: translate("ok_workout_send", lang),
                kind: NotificationKind::Temporal,
            });
            Ok(())
        }
        Err(e) => Err(report_error(
            e,
            lang,
            "error_workout_send",
            "Error sending workout to device",
        )),
    }
}

#[traced_command]
#[tauri::command]
pub async fn save_workout(
    app: AppHandle,
    settings: State<'_, SettingsLock>,
    workout: Workout,
) -> Result<(), String> {
    let name = workout.name.clone();
    let res = run_blocking(app, move |database| {
        database.run_in_transaction(|tx| {
            if WorkoutRepository::select_by_id_in(tx, &workout.name)?.is_some() {
                WorkoutStepRepository::delete()
                    .where_(Where::Eq(
                        workout_step::entity::columns::WORKOUT,
                        workout.name.clone().into(),
                    ))
                    .execute_in(tx)?;
            } else {
                let mut item = Workout {
                    name: workout.name.clone(),
                    enabled: true,
                    steps: vec![],
                };
                WorkoutRepository::insert().item(&mut item).execute_in(tx)?;
            }

            let mut insert = WorkoutStepRepository::insert();
            let mut steps = workout.steps.clone();
            for step in &mut steps {
                step.workout = workout.name.clone();
                insert = insert.item(step);
            }
            insert.execute_in(tx)?;

            Ok(())
        })
    })
    .await;

    let lang = settings.read().unwrap().language;
    match res {
        Ok(_) => {
            info!("Workout '{}' saved succesfully", name);
            show_notification(NotificationDefinition {
                title: name,
                body: translate("ok_workout_save", lang),
                kind: NotificationKind::Temporal,
            });
            Ok(())
        }
        Err(e) => Err(report_error(
            e,
            lang,
            "error_workout_save",
            "Error saving workout",
        )),
    }
}
