use std::{collections::HashMap, ops::Deref};

use garmin_tracker_rs_macros::{traced_command, translate};
use rusqlite_orm::{
    dao::{
        Repository,
        helpers::types::{order_by::OrderBy, value::Value, where_clause::Where},
    },
    database::{Database, errors::DatabaseError},
};
use tauri_plugin_log::log::{error, info};

use crate::{
    dao::{
        serie::{self, SerieRepository},
        session::{SessionRepository, entity},
    },
    dto::{
        notifications::{NotificationDefinition, NotificationKind},
        workouts::{WorkoutDetails, WorkoutListItem, WorkoutSession},
    },
    logic::notifications::show_notification,
    utils::date_time_utils::DateTimeUtils,
};

#[traced_command]
#[tauri::command]
pub fn get_workout_list() -> Result<Vec<WorkoutListItem>, String> {
    info!("Getting workouts list...");
    let res = Database::run_in_connection(|conn| {
        let sessions = SessionRepository::select()
            .order_by(OrderBy::Desc(entity::columns::DATE))
            .fetch_in(conn)?;

        let mut workout_stats = HashMap::new();
        sessions.iter().for_each(|s| {
            let entry = workout_stats
                .entry(s.workout.clone())
                .or_insert((0_u32, 0_f64, s.date));
            entry.0 += 1_u32;
            entry.1 += s.total_elapsed_time;
            entry.2 = if s.date > entry.2 { s.date } else { entry.2 };
        });

        let mut res = workout_stats
            .into_iter()
            .map(|wd| WorkoutListItem {
                name: wd.0,
                sessions: wd.1.0,
                avg_time: DateTimeUtils::format_duration((wd.1.1 / (wd.1.0 as f64)) as u64),
                latest_session: DateTimeUtils::format_time_date(wd.1.2),
            })
            .collect::<Vec<_>>();

        res.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(res)
    });
    match res {
        Ok(l) => {
            info!("Retreived {} workouts", l.len());
            Ok(l)
        }
        Err(DatabaseError::RunningOnConnection(e)) => {
            error!("Error getting workouts list: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_workout_list"),
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
pub fn get_workout_details(name: &str) -> Result<WorkoutDetails, String> {
    let res = Database::run_in_connection(|conn| {
        info!("Getting details for workout {}", name);

        let sessions = SessionRepository::select_by_workout_in_conn(
            conn,
            name,
            Some(&[OrderBy::Desc(entity::columns::DATE)]),
        )?;

        let mut latest = sessions.first().unwrap().clone();
        let mut count = 0_u32;
        let mut time = 0_f64;
        let mut volume = 0_f64;

        let series = SerieRepository::select()
            .where_(Where::In(
                serie::entity::columns::SESSION,
                sessions.iter().map(|s| Value::from(s.date)).collect(),
            ))
            .fetch_in(conn)?;

        let mut session_list = Vec::new();
        for session in &sessions {
            let session = session.clone();
            if session.date > latest.date {
                latest = session.clone();
            }
            count += 1;
            time += session.total_elapsed_time;

            let session_series = series.iter().filter(|s| s.session == session.date);
            let mut local_volume = 0_f64;
            for serie in session_series {
                local_volume += (serie.reps as f64) * serie.weight;
            }
            volume += local_volume;
            let mut wk_sess = WorkoutSession::from(&session);
            wk_sess.volume = local_volume;
            session_list.push(wk_sess);
        }

        let mut details = WorkoutDetails {
            name: name.to_string(),
            avg_time: DateTimeUtils::format_duration((time / (sessions.len() as f64)) as u64),
            latest_session: DateTimeUtils::format_time_date(latest.date),
            avg_volume: volume / (sessions.len() as f64),
            session_count: count,
            sessions: session_list,
        };

        for i in 0..details.sessions.len().saturating_sub(1) {
            let (left, right) = details.sessions.split_at_mut(i + 1);

            let current = &mut left[i];
            let previous = &right[0];

            current.vol_diff = format!(
                "{:+.2}%",
                (current.volume - previous.volume) / previous.volume * 100.0
            );
        }

        Ok(details)
    });

    match res {
        Ok(l) => {
            info!("Found details for workout {}", l.name);
            Ok(l)
        }
        Err(DatabaseError::RunningOnConnection(e)) => {
            error!("Error getting workout details: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_workout_details"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}
