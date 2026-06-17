use tauri::AppHandle;

use crate::{
    garmin::{
        database::DATABASE_INST,
        ui::{
            self, get_exercise_list, get_session_list, import_fit_file, show_exercise_details,
            update_session_sets,
        },
    },
    models::{
        exercises::{ExerciseDetails, ExerciseListItem},
        sessions::{SessionDetails, SessionListItem, SessionSeriesUpdate},
    },
};

mod garmin;
mod models;

#[tauri::command]
fn get_sessions() -> Result<Vec<SessionListItem>, String> {
    match get_session_list() {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("{}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_session_details(timestamp: i64) -> Result<SessionDetails, String> {
    match ui::get_session_details(timestamp) {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("{}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_exercises() -> Result<Vec<ExerciseListItem>, String> {
    match get_exercise_list() {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("{}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_exercise_details(category: &str, id: i16) -> Result<ExerciseDetails, String> {
    match show_exercise_details(category, id) {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("{}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn import_file(app: AppHandle) -> Result<SessionListItem, String> {
    match import_fit_file(app) {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("{}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn save_session_changes(details: SessionSeriesUpdate) -> Result<(), String> {
    update_session_sets(details)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_| {
            let config_dir = dirs::config_dir().expect("Could not get config folder");
            let db_dir = config_dir.join("taurfit");
            std::fs::create_dir_all(&db_dir).unwrap();
            let db_path = db_dir.join("taurfit.db");

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
            save_session_changes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
