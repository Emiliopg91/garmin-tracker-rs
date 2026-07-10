pub mod models;

use tauri_plugin_log::log::{error, info};

use crate::{
    garmin::database::dao::{exercise::Exercise, serie::Serie, session::Session},
    ui::{
        exercises::models::{ExerciseDetails, ExerciseListItem},
        notifications::{
            models::{NotificationDefinition, NotificationKind},
            show_notification,
        },
        sessions::models::SessionSerie,
        translations::{TRANSLATOR_INST, translation_keys::TranslationKeys},
    },
};

#[tauri::command]
pub fn get_exercises() -> Result<Vec<ExerciseListItem>, String> {
    info!("Getting exercises list...");
    let res: Result<Vec<ExerciseListItem>, String> = {
        let mut result = Vec::new();

        let exercises = Exercise::load_from_db().map_err(|e| e.to_string())?;
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
                title: TRANSLATOR_INST.translate(TranslationKeys::ERROR_EXERCISE_LIST),
                body: e.to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_exercise_details(category: &str, id: i16) -> Result<ExerciseDetails, String> {
    info!(
        "Getting details for exercise with category {} and id {}...",
        category, id
    );
    let res = {
        if let Some(exercise) =
            Exercise::load_by_cat_and_id(category, id as u16).map_err(|e| e.to_string())?
        {
            let mut res = ExerciseDetails::from(&exercise);

            let pr = Serie::get_pr_for_exercise(&exercise).map_err(|e| e.to_string())?;
            res.reps = pr.reps;
            res.weight = pr.weight;
            res.rm = pr.get_1rm_estimation();
            res.pr_date = pr.format_date();

            let series = Serie::load_for_exercise(category, id).map_err(|e| e.to_string())?;
            for serie in series {
                let wk = SessionSerie::from(&serie);
                if let Some(ses) = Session::find_by_id(serie.session.timestamp(), false)
                    .map_err(|e| e.to_string())?
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
                title: TRANSLATOR_INST.translate(TranslationKeys::ERROR_EXERCISE_DETAILS),
                body: e.to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}
