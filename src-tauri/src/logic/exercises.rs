use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

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
        exercise::{self, ExerciseRepository},
        serie::{self, SerieRepository},
        session::{self, SessionRepository},
    },
    dto::{
        exercises::{ExerciseDetails, ExerciseListItem},
        notifications::{NotificationDefinition, NotificationKind},
        sessions::SessionSerie,
    },
    logic::notifications::show_notification,
    utils::date_time_utils::DateTimeUtils,
};

#[traced_command]
#[tauri::command]
pub fn get_exercises() -> Result<Vec<ExerciseListItem>, String> {
    info!("Getting exercises list...");
    let res = Database::run_in_connection(|conn| {
        let mut result = Vec::new();

        let exercises = ExerciseRepository::select()
            .order_by(OrderBy::Asc(exercise::entity::columns::NAME))
            .fetch_in(conn)?;

        let prs = SerieRepository::select_by_personal_records_in_conn(conn, true, None)?;

        for exercise in exercises {
            let pr = prs
                .iter()
                .find(|pr| pr.ex_cat == exercise.category && pr.ex_id == exercise.id)
                .unwrap();

            result.push(ExerciseListItem {
                category: exercise.category,
                id: exercise.id,
                name: exercise.name,
                reps: pr.reps,
                weight: pr.weight,
                rm: get_1rm_estimation(pr.weight, pr.reps as f64),
                date: DateTimeUtils::format_time_date(pr.session),
            });
        }

        Ok(result)
    });

    match res {
        Ok(l) => {
            info!("Retreived {} exercises", l.len());
            Ok(l)
        }
        Err(DatabaseError::Transaction(e)) => {
            error!("Error getting exercises list: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_exercise_list"),
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
pub fn get_exercise_details(category: &str, id: u16) -> Result<ExerciseDetails, String> {
    info!(
        "Getting details for exercise with category {} and id {}...",
        category, id
    );
    let res = Database::run_in_connection(|conn| {
        let exercise = ExerciseRepository::select_by_id_in(conn, category, id)?.unwrap();
        let mut res = ExerciseDetails::from(&exercise);

        let series = SerieRepository::select_by_exercise_in_conn(
            conn,
            category,
            id,
            Some(&[OrderBy::Desc(serie::entity::columns::SESSION)]),
        )?;

        let pr = series.iter().filter(|s| s.pr).collect::<Vec<_>>();
        let pr = pr.first().unwrap();

        res.reps = pr.reps;
        res.weight = pr.weight;
        res.rm = get_1rm_estimation(pr.weight, pr.reps as f64);
        res.pr_date = DateTimeUtils::format_time_date(pr.session);

        let mut timestamps = HashSet::new();
        series.iter().map(|s| s.session).for_each(|t| {
            timestamps.insert(t);
        });

        let workouts = SessionRepository::select()
            .where_(Where::In(
                session::entity::columns::DATE,
                timestamps
                    .into_iter()
                    .map(|t| t.into())
                    .collect::<Vec<Value>>(),
            ))
            .fetch_in(conn)?
            .iter()
            .map(|s| (s.date, s.workout.clone()))
            .collect::<HashMap<_, _>>();

        for serie in series {
            let wk = SessionSerie::from(&serie);
            let ex_str = format!(
                "{}\n{}",
                workouts.get(&serie.session).unwrap(),
                DateTimeUtils::format_time_date(serie.session)
            );

            if !res.workouts.contains(&ex_str) {
                res.workouts.push(ex_str.clone());
            }

            let entry = res.series.entry(ex_str).or_default();
            entry.push(wk);
        }

        Ok(res)
    });

    match res {
        Ok(l) => {
            info!("Found details for exercise {}", l.name);
            Ok(l)
        }
        Err(DatabaseError::Transaction(e)) => {
            error!("Error getting exercise details: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_exercise_details"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

pub fn get_1rm_estimation(weight: f64, reps: f64) -> f64 {
    weight * reps.powf(0.1)
}
