use std::sync::mpsc;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::{
    garmin::database::{
        DATABASE_INST,
        dao::{exercise::Exercise, serie::Serie, session::Session},
    },
    models::{
        exercises::{ExerciseDetails, ExerciseListItem},
        workouts::{WorkoutDetails, WorkoutListItem, WorkoutSerie},
    },
};

pub fn get_session_list() -> Vec<WorkoutListItem> {
    let sessions = Session::load_from_db().unwrap();

    sessions
        .into_iter()
        .map(|s| WorkoutListItem::from(&s))
        .collect::<Vec<_>>()
}

pub fn get_session_details(timestamp: i64) -> WorkoutDetails {
    let session = Session::find_by_id(timestamp).unwrap().unwrap();

    let mut details = WorkoutDetails::from(&session);

    for (exercise, series) in &session.series {
        let entry = details.series.entry(exercise.name.clone()).or_default();
        for serie in series {
            entry.push(WorkoutSerie {
                reps: serie.reps,
                weight: serie.weight,
            });
        }
    }

    details
}

pub fn import_fit_file(app: AppHandle) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .add_filter("Garmin FIT file", &["fit"])
        .pick_file(move |file| {
            if let Some(file) = file {
                let _ = tx.send(file);
            }
        });

    let mut res = Ok(());
    match rx.recv() {
        Ok(file) => match Session::load_from_file(file.as_path().unwrap()) {
            Ok(session) => match DATABASE_INST.lock() {
                Ok(mut db) => {
                    match db.run_in_transaction(|tx| {
                        session.insert(tx)?;
                        Ok(())
                    }) {
                        Ok(_) => {}
                        Err(e) => {
                            res = Err(format!("Error writing to database: {}", e));
                        }
                    }
                }
                Err(e) => {
                    res = Err(format!("Error accesing to database: {}", e));
                }
            },
            Err(e) => res = Err(format!("Error parsing session: {}", e)),
        },
        Err(e) => res = Err(e.to_string()),
    }

    res
}

pub fn get_exercise_list() -> Vec<ExerciseListItem> {
    let mut result = Vec::new();

    let exercises = Exercise::load_from_db().unwrap();
    for exercise in exercises {
        let pr = Serie::get_pr_for_exercise(&exercise).unwrap();
        result.push(ExerciseListItem {
            category: exercise.category,
            id: exercise.id,
            name: exercise.name,
            reps: pr.reps,
            weight: pr.weight,
            rm: pr.get_1rm_estimation(),
        });
    }

    result
}

pub fn show_exercise_details(category: &str, id: i16) -> ExerciseDetails {
    let exercise = Exercise::load_by_cat_and_id(&category, id as u16)
        .unwrap()
        .unwrap();
    let mut res = ExerciseDetails::from(&exercise);

    let pr = Serie::get_pr_for_exercise(&exercise).unwrap();
    res.reps = pr.reps;
    res.weight = pr.weight;
    res.rm = pr.get_1rm_estimation();

    let series = Serie::load_for_exercise(category, id).unwrap();
    for serie in series {
        let wk = WorkoutSerie::from(&serie);
        let ses = Session::find_by_id(serie.session.timestamp())
            .unwrap()
            .unwrap();

        let entry = res
            .workouts
            .entry(format!("{}\n{}", ses.workout, ses.format_date()))
            .or_default();
        entry.push(wk);
    }

    res
}
