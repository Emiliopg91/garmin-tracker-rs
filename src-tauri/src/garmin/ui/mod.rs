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
        sessions::{SessionDetails, SessionListItem, SessionSerie, SessionSeriesUpdate},
    },
};

pub fn get_session_list() -> Result<Vec<SessionListItem>, String> {
    let sessions = Session::load_from_db().map_err(|e| e.to_string())?;

    Ok(sessions
        .into_iter()
        .map(|s| SessionListItem::from(&s))
        .collect::<Vec<_>>())
}

pub fn get_session_details(timestamp: i64) -> Result<SessionDetails, String> {
    if let Some(session) = Session::find_by_id(timestamp).map_err(|e| e.to_string())? {
        let mut details = SessionDetails::from(&session);

        for (exercise, series) in &session.series {
            if !details.exercises.contains(&exercise.name) {
                details.exercises.push(exercise.name.clone())
            }
            let entry = details.series.entry(exercise.name.clone()).or_default();
            for serie in series {
                entry.push(SessionSerie::from(serie));
            }
        }

        Ok(details)
    } else {
        Err("Could not find session".to_string())
    }
}

pub fn import_fit_file(app: AppHandle) -> Result<SessionListItem, String> {
    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .add_filter("Garmin FIT file", &["fit"])
        .pick_files(move |file| {
            if let Some(file) = file {
                let _ = tx.send(file);
            }
        });

    let mut res = Ok(SessionListItem::default());
    match rx.recv() {
        Ok(files) => match DATABASE_INST.lock() {
            Ok(mut db) => {
                match db.run_in_transaction(|tx| {
                    for file in &files {
                        match Session::load_from_file(file.as_path().unwrap()) {
                            Ok(mut session) => {
                                session.insert(tx)?;
                                res = Ok(SessionListItem::from(&session))
                            }
                            Err(e) => {
                                res = Err(format!("Error parsing session: {}", e));
                                break;
                            }
                        }
                    }
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
        Err(e) => res = Err(e.to_string()),
    }

    res
}

pub fn get_exercise_list() -> Result<Vec<ExerciseListItem>, String> {
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
        });
    }

    Ok(result)
}

pub fn show_exercise_details(category: &str, id: i16) -> Result<ExerciseDetails, String> {
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
            if let Some(ses) =
                Session::find_by_id(serie.session.timestamp()).map_err(|e| e.to_string())?
            {
                let ex_str = format!("{}\n{}", ses.workout, ses.format_date());

                if !res.workouts.contains(&ex_str) {
                    res.workouts.push(ex_str.clone());
                }

                let entry = res.series.entry(ex_str).or_default();
                entry.push(wk);
            } else {
                return Err("Could not find session".to_string());
            }
        }

        Ok(res)
    } else {
        Err("Could not find exercise".to_string())
    }
}

pub fn update_session_sets(details: SessionSeriesUpdate) -> Result<(), String> {
    let mut to_update = Vec::new();
    for serie in details.series {
        let db_serie = Serie::load_for_session_and_idx(details.timestamp, serie.idx)
            .map_err(|e| e.to_string())?;
        if let Some(mut db_serie) = db_serie {
            db_serie.reps = serie.reps;
            db_serie.weight = serie.weight;
            to_update.push(db_serie);
        }
    }
    let exercises = Exercise::load_from_db().map_err(|e| e.to_string())?;

    let mut db = DATABASE_INST.lock().map_err(|e| e.to_string())?;
    db.run_in_transaction(move |tx| {
        for to_upd in &to_update {
            to_upd.update_serie(tx);
        }
        for exer in &exercises {
            Serie::update_pr(tx, &exer.category, exer.id);
        }
        Ok(())
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}
