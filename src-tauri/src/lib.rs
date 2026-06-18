use tauri::{AppHandle, Manager, image::Image};

use crate::garmin::{
    database::DATABASE_INST,
    models::{
        exercises::{ExerciseDetails, ExerciseListItem},
        sessions::{SessionDetails, SessionListItem, SessionSeriesUpdate},
        workouts::{WorkoutDetails, WorkoutListItem},
    },
    ui::{
        self, get_exercise_list, get_session_list, import_fit_file, show_exercise_details,
        update_session_sets,
    },
};

mod garmin;

#[tauri::command]
fn get_sessions() -> Result<Vec<SessionListItem>, String> {
    get_session_list()
}

#[tauri::command]
fn get_session_details(timestamp: i64) -> Result<SessionDetails, String> {
    ui::get_session_details(timestamp)
}

#[tauri::command]
fn get_exercises() -> Result<Vec<ExerciseListItem>, String> {
    get_exercise_list()
}

#[tauri::command]
fn get_exercise_details(category: &str, id: i16) -> Result<ExerciseDetails, String> {
    show_exercise_details(category, id)
}

#[tauri::command]
async fn import_file(app: AppHandle) -> Result<SessionListItem, String> {
    import_fit_file(app)
}

#[tauri::command]
fn save_session_changes(details: SessionSeriesUpdate) -> Result<(), String> {
    update_session_sets(details)
}

#[tauri::command]
fn get_workout_list() -> Result<Vec<WorkoutListItem>, String> {
    ui::get_workout_list()
}

#[tauri::command]
fn get_workout_details(name: &str) -> Result<WorkoutDetails, String> {
    ui::get_workout_details(name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_| {
            let config_dir = dirs::config_dir().expect("Could not get config folder");
            let db_dir = config_dir.join("garmin-fit-rs");
            std::fs::create_dir_all(&db_dir).unwrap();
            let db_path = db_dir.join("database.db");

            let mut db = DATABASE_INST.lock().unwrap();
            db.open(db_path).unwrap();
            db.create_schema().unwrap();

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_exercises,
            get_session_details,
            get_exercise_details,
            import_file,
            save_session_changes,
            get_workout_list,
            get_workout_details
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
