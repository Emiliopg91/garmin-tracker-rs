use crate::{
    garmin::{
        database::DATABASE_INST,
        ui::{get_exercise_list, get_record_list, get_session_list},
    },
    models::{exercises::ExerciseListItem, records::RecordListItem, workouts::WorkoutListItem},
};

mod garmin;
mod models;

#[tauri::command]
fn get_workouts() -> Vec<WorkoutListItem> {
    get_session_list()
}

#[tauri::command]
fn get_exercises() -> Vec<ExerciseListItem> {
    get_exercise_list()
}

#[tauri::command]
fn get_records() -> Vec<RecordListItem> {
    get_record_list()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_| {
            let mut db = DATABASE_INST.lock().unwrap();
            db.open("./db.sqlite").unwrap();
            db.create_schema().unwrap();

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_workouts,
            get_exercises,
            get_records
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
