use std::collections::HashMap;

use garmin_tracker_rs_macros::traced_command;
use rusqlite_orm::{
    dao::Repository,
    database::DatabasePool,
    types::{order_by::OrderBy, value::Value, where_clause::Where},
};
use tauri::State;
use tauri_plugin_log::log::info;

use crate::{
    SettingsLock,
    dao::{
        exercise::ExerciseRepository,
        session::{self, SessionRepository},
        set::{self, SetRepository},
    },
    dto::{
        exercises::{ExerciseDetails, ExerciseListItem},
        sessions::SessionSet,
    },
    logic::report_error,
};

/// Returns every exercise in the catalog, each annotated with its current personal record.
#[traced_command]
#[tauri::command]
pub fn get_exercises(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
) -> Result<Vec<ExerciseListItem>, String> {
    info!("Getting exercises list...");
    let res = database.run_in_connection(|conn| {
        let prs = SetRepository::select_by_personal_records_in(conn, true, None)?;

        Ok(prs
            .iter()
            .map(|pr| ExerciseListItem {
                category: pr.ex_cat,
                id: pr.ex_id,
                reps: pr.reps,
                weight: pr.weight,
                date: pr.session as i32,
                e1rm: pr.e1rm,
            })
            .collect::<Vec<_>>())
    });

    match res {
        Ok(l) => {
            info!("Retreived {} exercises", l.len());
            Ok(l)
        }
        Err(e) => Err(report_error(
            e,
            settings.read().unwrap().language,
            "error_exercise_list",
            "Error getting exercises list",
        )),
    }
}

/// Returns the personal record and full per-session set history for one exercise.
#[traced_command]
#[tauri::command]
pub fn get_exercise_details(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
    category: u16,
    id: u16,
) -> Result<ExerciseDetails, String> {
    info!(
        "Getting details for exercise with category {} and id {}...",
        category, id
    );
    let res = database.run_in_connection(|conn| {
        let exercise = ExerciseRepository::select_by_id_in(conn, category, id)?.unwrap();
        let mut res = ExerciseDetails::from(&exercise);

        let series = SetRepository::select_by_exercise_in(
            conn,
            category,
            id,
            Some(&[
                OrderBy::Desc(set::entity::columns::SESSION),
                OrderBy::Asc(set::entity::columns::IDX),
            ]),
        )?;

        let pr = series.iter().find(|s| s.pr).unwrap();
        res.reps = pr.reps;
        res.weight = pr.weight;
        res.pr_date = pr.session as i32;
        res.e1rm = pr.e1rm;

        let mut timestamps: Vec<Value> = Vec::new();
        let mut last = None;
        for s in &series {
            if last != Some(s.session) {
                timestamps.push(s.session.into());
                last = Some(s.session);
            }
        }

        let workouts = SessionRepository::select()
            .where_(Where::In(session::entity::columns::DATE, timestamps))
            .fetch_in(conn)?
            .iter()
            .map(|s| (s.date, s.name.clone()))
            .collect::<HashMap<_, _>>();

        let mut last_session = None;
        let mut ex_str = String::new();
        for serie in series {
            if last_session != Some(serie.session) {
                ex_str = format!("{}\n{}", workouts.get(&serie.session).unwrap(), serie.session);
                res.workouts.push(ex_str.clone());
                last_session = Some(serie.session);
            }

            let entry = res.series.entry(ex_str.clone()).or_default();
            entry.push(SessionSet::from(&serie));
        }

        Ok(res)
    });

    match res {
        Ok(l) => {
            info!("Found details for exercise {} - {}", category, id);
            Ok(l)
        }
        Err(e) => Err(report_error(
            e,
            settings.read().unwrap().language,
            "error_exercise_details",
            "Error getting exercise details",
        )),
    }
}
