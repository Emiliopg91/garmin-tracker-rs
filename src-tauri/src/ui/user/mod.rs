use garmin_tracker_rs_macros::traced_command;
use tauri_plugin_log::log::{error, info};

use crate::{
    garmin::database::dao::user::User,
    ui::{
        notifications::{
            models::{NotificationDefinition, NotificationKind},
            show_notification,
        },
        translations::TRANSLATOR_INST,
        user::models::UserListItem,
    },
};

pub mod models;

#[traced_command]#[tauri::command]
pub fn get_user_measures() -> Result<Vec<UserListItem>, String> {
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
            show_notification(NotificationDefinition {
                title: TRANSLATOR_INST.translate("error_measures_list"),
                body: e.clone(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}

#[traced_command]#[tauri::command]
pub fn add_user_measures(measures: UserListItem) -> Result<(), String> {
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
            show_notification(NotificationDefinition {
                title: TRANSLATOR_INST.translate("error_adding_measures"),
                body: e.clone(),
                kind: NotificationKind::Persistant,
            });
            Err(e)
        }
    }
}
