pub mod models;

use garmin_tracker_rs_macros::{traced_command, translate};
use tauri_plugin_log::log::{error, info};

use crate::{
    garmin::database::dao::{
        Entity,
        exercise::{EXERCISE_COLUMN_NAME, Exercise},
        helpers::types::{order_by::OrderBy, where_clause::Where},
        serie::{
            SERIE_COLUMN_EXERCISE_CATEGORY, SERIE_COLUMN_EXERCISE_ID, SERIE_COLUMN_SESSION, Serie,
        },
        session::Session,
    },
    ui::{
        exercises::models::{ExerciseDetails, ExerciseListItem},
        notifications::{
            models::{NotificationDefinition, NotificationKind},
            show_notification,
        },
        sessions::models::SessionSerie,
    },
};

#[traced_command]
#[tauri::command]
pub fn get_exercises() -> Result<Vec<ExerciseListItem>, String> {
    info!("Getting exercises list...");
    let res: Result<Vec<ExerciseListItem>, String> = {
        let mut result = Vec::new();

        let exercises = Exercise::select()
            .order_by(OrderBy::Asc(EXERCISE_COLUMN_NAME))
            .fetch()
            .map_err(|e| e.to_string())?;
        for exercise in exercises {
            let pr = Serie::get_pr_for_exercise(&exercise).map_err(|e| e.to_string())?;
            result.push(ExerciseListItem {
                category: exercise.category,
                id: exercise.id,
                name: exercise.name,
                reps: pr.reps,
                weight: pr.weight,
                rm: pr.get_1rm_estimation(),
                date: pr.format_date(),
            });
        }

        Ok(result)
    };

    match res {
        Ok(l) => {
            info!("Retreived {} exercises", l.len());
            Ok(l)
        }
        Err(e) => {
            error!("Error getting exercises list: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_exercise_list"),
                body: e.to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

#[traced_command]
#[tauri::command]
pub fn get_exercise_details(category: &str, id: i16) -> Result<ExerciseDetails, String> {
    info!(
        "Getting details for exercise with category {} and id {}...",
        category, id
    );
    let res = {
        if let Some(exercise) =
            Exercise::select_by_id(category.to_string(), id as u16).map_err(|e| e.to_string())?
        {
            let mut res = ExerciseDetails::from(&exercise);

            let pr = Serie::get_pr_for_exercise(&exercise).map_err(|e| e.to_string())?;
            res.reps = pr.reps;
            res.weight = pr.weight;
            res.rm = pr.get_1rm_estimation();
            res.pr_date = pr.format_date();

            let series = Serie::select()
                .where_(Where::And(vec![
                    Where::Eq(SERIE_COLUMN_EXERCISE_CATEGORY, category.into()),
                    Where::Eq(SERIE_COLUMN_EXERCISE_ID, id.into()),
                ]))
                .order_by(OrderBy::Desc(SERIE_COLUMN_SESSION))
                .fetch()
                .map_err(|e| e.to_string())?;
            for serie in series {
                let wk = SessionSerie::from(&serie);
                if let Some(ses) =
                    Session::find_by_id(serie.session, false).map_err(|e| e.to_string())?
                {
                    let ex_str = format!("{}\n{}", ses.workout, ses.format_date());

                    if !res.workouts.contains(&ex_str) {
                        res.workouts.push(ex_str.clone());
                    }

                    let entry = res.series.entry(ex_str).or_default();
                    entry.push(wk);
                }
            }

            Ok(res)
        } else {
            Err("Could not find exercise".to_string())
        }
    };

    match res {
        Ok(l) => {
            info!("Found details for exercise {}", l.name);
            Ok(l)
        }
        Err(e) => {
            error!("Error getting exercise details: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_exercise_details"),
                body: e.to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}
