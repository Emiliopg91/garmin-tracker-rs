use tauri::AppHandle;

use crate::garmin::{
    database::DATABASE_INST,
    models::{
        devices::DeviceListItem,
        exercises::{ExerciseDetails, ExerciseListItem},
        notifications::NotificationDefinition,
        sessions::{SessionDetails, SessionListItem, SessionSeriesUpdate},
        workouts::{WorkoutDetails, WorkoutListItem},
    },
    ui,
};

mod garmin;

#[tauri::command]
fn get_sessions() -> Result<Vec<SessionListItem>, String> {
    ui::get_session_list()
}

#[tauri::command]
fn get_session_details(timestamp: i64) -> Result<SessionDetails, String> {
    ui::get_session_details(timestamp)
}

#[tauri::command]
fn get_exercises() -> Result<Vec<ExerciseListItem>, String> {
    ui::get_exercise_list()
}

#[tauri::command]
fn get_exercise_details(category: &str, id: i16) -> Result<ExerciseDetails, String> {
    ui::show_exercise_details(category, id)
}

#[tauri::command]
async fn import_from_file(app: AppHandle) -> Result<isize, String> {
    ui::import_fit_file(app)
}

#[tauri::command]
async fn import_from_device(serial: &str) -> Result<usize, String> {
    ui::import_from_device(serial).await
}

#[tauri::command]
fn save_session_changes(details: SessionSeriesUpdate) -> Result<(), String> {
    ui::update_session_sets(details)
}

#[tauri::command]
fn get_workout_list() -> Result<Vec<WorkoutListItem>, String> {
    ui::get_workout_list()
}

#[tauri::command]
fn get_workout_details(name: &str) -> Result<WorkoutDetails, String> {
    ui::get_workout_details(name)
}

#[tauri::command]
async fn get_available_devices() -> Result<Vec<DeviceListItem>, String> {
    ui::get_available_devices().await
}

#[tauri::command]
async fn show_notification(
    app: AppHandle,
    notification: NotificationDefinition,
) -> Result<(), String> {
    ui::show_notification(app, notification)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |_| {
            let config_dir = dirs::config_dir().expect("Could not get config folder");
            let db_dir = config_dir.join("garmin-tracker-rs");
            std::fs::create_dir_all(&db_dir).unwrap();
            let db_path = db_dir.join("database.db");

            let mut db = DATABASE_INST.lock().unwrap();
            db.open(db_path).unwrap();
            db.create_schema().unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_exercises,
            get_session_details,
            get_exercise_details,
            import_from_file,
            save_session_changes,
            get_workout_list,
            get_workout_details,
            get_available_devices,
            import_from_device,
            show_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
