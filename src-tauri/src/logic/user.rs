use std::ops::Deref;

use garmin_tracker_rs_macros::{traced_command, translate};
use rusqlite_orm::{
    dao::{Repository, helpers::types::order_by::OrderBy},
    database::{DATABASE_INST, errors::DatabaseError},
};
use tauri_plugin_log::log::{error, info};

use crate::{
    dao::user::{self, User, UserRepository},
    dto::{
        notifications::{NotificationDefinition, NotificationKind},
        user::UserListItem,
    },
    logic::notifications::show_notification,
};

#[traced_command]
#[tauri::command]
pub fn get_user_measures() -> Result<Vec<UserListItem>, String> {
    info!("Getting user measures list...");

    let res = DATABASE_INST.lock().unwrap().run_in_tx(|tx| {
        let regs = UserRepository::select()
            .order_by(OrderBy::Desc(user::entity::columns::DATE))
            .fetch_in_tx(tx)?;

        Ok(regs)
    });

    match res {
        Ok(regs) => {
            let res = regs
                .iter()
                .map(UserListItem::from)
                .collect::<Vec<UserListItem>>();

            info!("Retrieved {} measures", res.len());
            Ok(res)
        }
        Err(DatabaseError::Transaction(e)) => {
            error!("Error getting measures list: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_measures_list"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}

#[traced_command]
#[tauri::command]
pub fn add_user_measures(measures: UserListItem) -> Result<(), String> {
    info!("Adding user measures list...");

    let res = DATABASE_INST.lock().unwrap().run_in_tx(|tx| {
        let entry = User::try_from(&measures).map_err(DatabaseError::Transaction)?;

        UserRepository::insert().item(entry).execute_in_tx(tx)?;

        Ok(())
    });

    match res {
        Ok(_) => {
            info!("Measures added succesfully");
            Ok(())
        }
        Err(DatabaseError::Transaction(e)) => {
            error!("Error adding measures: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_adding_measures"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}
