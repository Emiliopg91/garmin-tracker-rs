use std::collections::HashMap;

use garmin_tracker_rs_macros::traced_command;
use rusqlite_orm::{
    dao::Repository,
    database::DatabasePool,
    types::{order_by::OrderBy, value::Value, where_clause::Where},
};
use tauri::State;
use tauri_plugin_log::log::{error, info};

use crate::{
    SettingsLock,
    dao::{
        serie::{self, SerieRepository},
        session::{self, SessionRepository, entity},
    },
    dto::{
        notifications::{NotificationDefinition, NotificationKind},
        workouts::{WorkoutDetails, WorkoutListItem, WorkoutSession},
    },
    logic::notifications::show_notification,
    utils::translations::translate,
};

/// Returns sessions grouped/aggregated by workout name (count, average time, latest date), sorted by name.
#[traced_command]
#[tauri::command]
pub fn get_workout_list(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
) -> Result<Vec<WorkoutListItem>, String> {
    info!("Getting workouts list...");
    let res = database.run_in_connection(|conn| {
        let subquery = SerieRepository::select().distinct(&[serie::entity::columns::SESSION]);

        let sessions = SessionRepository::select()
            .where_(Where::InSub(
                session::entity::columns::DATE,
                subquery.to_subquery(),
            ))
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
                avg_time: (wd.1.1 / (wd.1.0 as f64)).round() as i32,
                latest_session: wd.1.2 as i32,
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
        Err(e) => {
            error!("Error getting workouts list: {}", e);
            show_notification(NotificationDefinition {
                title: translate("error_workout_list", settings.read().unwrap().language),
                body: e.to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.to_string())
        }
    }
}

/// Returns every session for one workout name with per-session volume and session-over-session volume diff.
#[traced_command]
#[tauri::command]
pub fn get_workout_details(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
    name: &str,
) -> Result<WorkoutDetails, String> {
    let res = database.run_in_connection(|conn| {
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

        let mut volume_by_session: HashMap<i64, f64> = HashMap::new();
        for serie in &series {
            *volume_by_session.entry(serie.session).or_insert(0.0) +=
                (serie.reps as f64) * serie.weight;
        }

        let mut session_list = Vec::new();
        for session in &sessions {
            if session.date > latest.date {
                latest = session.clone();
            }
            count += 1;
            time += session.total_elapsed_time;

            let local_volume = volume_by_session.get(&session.date).copied().unwrap_or(0.0);
            volume += local_volume;
            let mut wk_sess = WorkoutSession::from(session);
            wk_sess.volume = local_volume;
            session_list.push(wk_sess);
        }

        let mut details = WorkoutDetails {
            name: name.to_string(),
            avg_time: (time / (sessions.len() as f64)).round() as i32,
            latest_session: latest.date as i32,
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
        Err(e) => {
            error!("Error getting workout details: {}", e);
            show_notification(NotificationDefinition {
                title: translate("error_workout_details", settings.read().unwrap().language),
                body: e.to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.to_string())
        }
    }
}
