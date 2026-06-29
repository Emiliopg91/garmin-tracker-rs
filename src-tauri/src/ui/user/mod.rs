use tauri::AppHandle;
use tauri_plugin_log::log::{error, info};

use crate::{
    garmin::database::dao::user::User,
    ui::{
        notifications::{models::NotificationDefinition, show_notification},
        user::models::UserListItem,
    },
};

pub mod models;

#[tauri::command]
pub fn get_user_measures(app: AppHandle) -> Result<Vec<UserListItem>, String> {
    info!("Getting user measures list...");

    match User::select_all().map_err(|e| e.to_string()) {
        Ok(regs) => {
            let res = regs
                .iter()
                .map(UserListItem::from)
                .collect::<Vec<UserListItem>>();

            info!("Retrieved {} measures", res.len());
            Ok(res)
        }
        Err(e) => {
            error!("Error getting measures list: {}", e);
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error getting measures list".to_string(),
                    body: e.clone(),
                },
            );
            Err(e)
        }
    }
}

#[tauri::command]
pub fn add_user_measures(app: AppHandle, measures: UserListItem) -> Result<(), String> {
    info!("Adding user measures list...");
    dbg!(&measures);

    let res = match User::try_from(&measures).map_err(|e| e.to_string()) {
        Ok(entry) => entry.insert().map_err(|e| e.to_string()),
        Err(e) => Err(e),
    };

    match res {
        Ok(_) => {
            info!("Measures added succesfully");
            Ok(())
        }
        Err(e) => {
            error!("Error adding measures: {}", e);
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error adding measures".to_string(),
                    body: e.clone(),
                },
            );
            Err(e)
        }
    }
}
