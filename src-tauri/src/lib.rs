#[cfg(debug_assertions)]
use tauri::AppHandle;

use crate::{
    garmin::{
        database::DATABASE_INST,
        ui::{
            get_exercise_list, get_session_details, get_session_list, import_fit_file,
            show_exercise_details, update_workout_sets,
        },
    },
    models::{
        exercises::{ExerciseDetails, ExerciseListItem},
        workouts::{WorkoutDetails, WorkoutListItem, WorkoutSeriesUpdate},
    },
};
use tauri_specta::*;

mod garmin;
mod models;

#[tauri::command]
#[specta::specta]
fn get_workouts() -> Vec<WorkoutListItem> {
    get_session_list()
}

#[tauri::command]
#[specta::specta]
fn get_workout_details(timestamp: i64) -> WorkoutDetails {
    get_session_details(timestamp)
}

#[tauri::command]
#[specta::specta]
fn get_exercises() -> Vec<ExerciseListItem> {
    get_exercise_list()
}

#[tauri::command]
#[specta::specta]
fn get_exercise_details(category: &str, id: i16) -> ExerciseDetails {
    show_exercise_details(category, id)
}

#[tauri::command]
#[specta::specta]
async fn import_file(app: AppHandle) -> Result<(), String> {
    import_fit_file(app)
}

#[tauri::command]
#[specta::specta]
fn save_workout_changes(details: WorkoutSeriesUpdate) {
    update_workout_sets(details);
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    {
        use specta::Types;
        use specta_typescript::Typescript;

        let types = Types::default()
            .register::<WorkoutListItem>()
            .register::<WorkoutDetails>()
            .register::<ExerciseListItem>()
            .register::<ExerciseDetails>()
            .register::<WorkoutSeriesUpdate>();

        Typescript::default()
            .export_to("../src/models/RpcModels.ts", &types, specta_serde::Format)
            .unwrap();
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_| {
            let mut db = DATABASE_INST.lock().unwrap();
            db.open("../db.sqlite").unwrap();
            db.create_schema().unwrap();

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_workouts,
            get_exercises,
            get_workout_details,
            get_exercise_details,
            import_file,
            save_workout_changes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
