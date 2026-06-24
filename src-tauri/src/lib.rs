use tauri::Manager;

use crate::{
    garmin::database::DATABASE_INST,
    ui::{
        devices::mtp_watcher,
        exercises::{get_exercise_details, get_exercises},
        sessions::{
            get_session_details, get_sessions, import_from_device, import_from_file,
            save_session_changes,
        },
        workouts::{get_workout_details, get_workout_list},
    },
};

mod garmin;
mod ui;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let config_dir = dirs::config_dir().expect("Could not get config folder");
            let db_dir = config_dir.join("garmin-tracker-rs");
            std::fs::create_dir_all(&db_dir).unwrap();
            let db_path = db_dir.join("database.db");

            let mut db = DATABASE_INST.lock().unwrap();
            db.open(db_path).unwrap();
            db.create_schema().unwrap();

            mtp_watcher(app.handle().clone());

            let window = app.get_webview_window("main").unwrap();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let _ = window.show();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_session_details,
            save_session_changes,
            get_exercises,
            get_exercise_details,
            import_from_file,
            get_workout_list,
            get_workout_details,
            import_from_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
