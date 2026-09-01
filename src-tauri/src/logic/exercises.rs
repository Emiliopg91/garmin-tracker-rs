use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::RwLock,
};

use garmin_tracker_rs_macros::traced_command;
use rusqlite_orm::{
    dao::Repository,
    database::DatabasePool,
    errors::DatabaseError,
    types::{order_by::OrderBy, value::Value, where_clause::Where},
};
use tauri::State;
use tauri_plugin_log::log::{error, info};

use crate::{
    dao::{
        exercise::{self, ExerciseRepository},
        serie::{self, Serie, SerieRepository},
        session::{self, SessionRepository},
    },
    dto::{
        app::Settings,
        exercises::{ExerciseDetails, ExerciseListItem},
        notifications::{NotificationDefinition, NotificationKind},
        sessions::SessionSerie,
    },
    logic::notifications::show_notification,
    utils::translations::translate,
};

/// Returns every exercise in the catalog, each annotated with its current personal record.
#[traced_command]
#[tauri::command]
pub fn get_exercises(
    database: State<'_, DatabasePool>,
    settings: State<'_, RwLock<Settings>>,
) -> Result<Vec<ExerciseListItem>, String> {
    info!("Getting exercises list...");
    let res = database.run_in_connection(|conn| {
        let mut result = Vec::new();

        let exercises = ExerciseRepository::select()
            .order_by(OrderBy::Asc(exercise::entity::columns::NAME))
            .fetch_in(conn)?;

        let prs = SerieRepository::select_by_personal_records_in_conn(conn, true, None)?;
        let pr_by_exercise: HashMap<(String, u16), &Serie> = prs
            .iter()
            .map(|pr| ((pr.ex_cat.clone(), pr.ex_id), pr))
            .collect();

        for exercise in exercises {
            let key = (exercise.category.clone(), exercise.id);
            let pr = pr_by_exercise[&key];

            result.push(ExerciseListItem {
                category: exercise.category,
                id: exercise.id,
                name: exercise.name,
                reps: pr.reps,
                weight: pr.weight,
                rm: get_1rm_estimation(pr.weight, pr.reps as f64),
                date: pr.session as i32,
            });
        }

        Ok(result)
    });

    match res {
        Ok(l) => {
            info!("Retreived {} exercises", l.len());
            Ok(l)
        }
        Err(DatabaseError::RunningOnConnection(e)) => {
            error!("Error getting exercises list: {}", e);
            show_notification(NotificationDefinition {
                title: translate("error_exercise_list", settings.read().unwrap().language),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

/// Returns the personal record and full per-session set history for one exercise.
#[traced_command]
#[tauri::command]
pub fn get_exercise_details(
    database: State<'_, DatabasePool>,
    settings: State<'_, RwLock<Settings>>,
    category: &str,
    id: u16,
) -> Result<ExerciseDetails, String> {
    info!(
        "Getting details for exercise with category {} and id {}...",
        category, id
    );
    let res = database.run_in_connection(|conn| {
        let exercise = ExerciseRepository::select_by_id_in(conn, category, id)?.unwrap();
        let mut res = ExerciseDetails::from(&exercise);

        let series = SerieRepository::select_by_exercise_in_conn(
            conn,
            category,
            id,
            Some(&[
                OrderBy::Desc(serie::entity::columns::SESSION),
                OrderBy::Asc(serie::entity::columns::IDX),
            ]),
        )?;

        let pr = series.iter().find(|s| s.pr).unwrap();
        res.reps = pr.reps;
        res.weight = pr.weight;
        res.rm = get_1rm_estimation(pr.weight, pr.reps as f64);
        res.pr_date = pr.session as i32;

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

        let mut last_session = None;
        for serie in series {
            let wk = SessionSerie::from(&serie);
            let ex_str = format!(
                "{}\n{}",
                workouts.get(&serie.session).unwrap(),
                serie.session
            );

            if last_session != Some(serie.session) {
                res.workouts.push(ex_str.clone());
                last_session = Some(serie.session);
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
        Err(DatabaseError::RunningOnConnection(e)) => {
            error!("Error getting exercise details: {}", e);
            show_notification(NotificationDefinition {
                title: translate("error_exercise_details", settings.read().unwrap().language),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

/// Estimates a 1-rep max from a weight/reps pair.
pub fn get_1rm_estimation(weight: f64, reps: f64) -> f64 {
    weight * reps.powf(0.1)
}
