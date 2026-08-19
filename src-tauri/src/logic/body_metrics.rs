use std::ops::Deref;

use garmin_tracker_rs_macros::{traced_command, translate};
use rusqlite_orm::{
    dao::{Repository, helpers::types::order_by::OrderBy},
    database::{Database, errors::DatabaseError},
};
use tauri_plugin_log::log::{error, info};

use crate::{
    dao::body_metrics::{self, BodyMetrics, BodyMetricsRepository},
    dto::{
        body_metrics::BodyMetricListItem,
        notifications::{NotificationDefinition, NotificationKind},
    },
    logic::notifications::show_notification,
};

#[traced_command]
#[tauri::command]
pub fn get_body_measures() -> Result<Vec<BodyMetricListItem>, String> {
    info!("Getting body measures list...");

    let res = Database::run_in_connection(|conn| {
        let regs = BodyMetricsRepository::select()
            .order_by(OrderBy::Desc(body_metrics::entity::columns::DATE))
            .fetch_in(conn)?;

        Ok(regs)
    });

    match res {
        Ok(regs) => {
            let res = regs
                .iter()
                .map(BodyMetricListItem::from)
                .collect::<Vec<BodyMetricListItem>>();

            info!("Retrieved {} measures", res.len());
            Ok(res)
        }
        Err(DatabaseError::RunningOnConnection(e)) => {
            error!("Error getting measures list: {}", e);
            show_notification(NotificationDefinition {
                title: translate!("error_body_measures_list"),
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
pub fn add_body_measures(measures: BodyMetricListItem) -> Result<(), String> {
    info!("Adding body measures list...");

    let res = Database::run_in_transaction(|tx| {
        let entry = BodyMetrics::try_from(&measures).map_err(DatabaseError::Transaction)?;

        BodyMetricsRepository::insert().item(entry).execute_in(tx)?;

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
                title: translate!("error_adding_body_measures"),
                body: e.deref().to_string(),
                kind: NotificationKind::Persistant,
            });
            Err(e.deref().to_string())
        }
        _ => unreachable!(),
    }
}
