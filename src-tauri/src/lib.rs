use tauri::AppHandle;

use crate::{
    garmin::{
        database::DATABASE_INST,
        ui::{
            get_exercise_list, get_session_details, get_session_list, import_fit_file,
            show_exercise_details,
        },
    },
    models::{
        exercises::{ExerciseDetails, ExerciseListItem},
        workouts::{WorkoutDetails, WorkoutListItem},
    },
};

mod garmin;
mod models;

#[tauri::command]
fn get_workouts() -> Vec<WorkoutListItem> {
    get_session_list()
}

#[tauri::command]
fn get_workout_details(timestamp: i64) -> WorkoutDetails {
    get_session_details(timestamp)
}

#[tauri::command]
fn get_exercises() -> Vec<ExerciseListItem> {
    get_exercise_list()
}

#[tauri::command]
fn get_exercise_details(category: &str, id: i16) -> ExerciseDetails {
    show_exercise_details(category, id)
}

#[tauri::command]
async fn import_file(app: AppHandle) -> Result<(), String> {
    import_fit_file(app).unwrap();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            get_workout_details,
            get_exercise_details,
            import_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
