use garmin_tracker_rs_macros::traced_command;
use rusqlite_orm::{
    dao::Repository, database::DatabasePool, errors::DatabaseError, types::order_by::OrderBy,
};
use tauri::State;
use tauri_plugin_log::log::info;

use crate::{
    SettingsLock,
    dao::body_metrics::{self, BodyMetrics, BodyMetricsRepository},
    dto::{
        body_metrics::BodyMetricListItem,
        notifications::{NotificationDefinition, NotificationKind},
    },
    logic::{notifications::show_notification, report_error},
    utils::translations::translate,
};

/// Returns all logged body measurements, newest first.
#[traced_command]
#[tauri::command]
pub fn get_body_measures(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
) -> Result<Vec<BodyMetricListItem>, String> {
    info!("Getting body measures list...");

    let res = database.run_in_connection(|conn| {
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
        Err(e) => Err(report_error(
            e,
            settings.read().unwrap().language,
            "error_body_measures_list",
            "Error getting measures list",
        )),
    }
}

/// Inserts a new body measurement entry.
#[traced_command]
#[tauri::command]
pub fn add_body_measures(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
    measures: BodyMetricListItem,
) -> Result<(), String> {
    info!("Adding body measures list...");

    let res = database.run_in_transaction(|tx| {
        let entry = BodyMetrics::try_from(&measures).map_err(DatabaseError::Transaction)?;

        BodyMetricsRepository::insert().item(entry).execute_in(tx)?;

        Ok(())
    });

    match res {
        Ok(_) => {
            info!("Measures added succesfully");
            Ok(())
        }
        Err(e) => Err(report_error(
            e,
            settings.read().unwrap().language,
            "error_adding_body_measures",
            "Error adding measures",
        )),
    }
}

/// Deletes the body measurement logged on `date`, if any.
#[traced_command]
#[tauri::command]
pub fn delete_body_metric(
    database: State<'_, DatabasePool>,
    settings: State<'_, SettingsLock>,
    date: i32,
) -> Result<(), String> {
    let res = database.run_in_transaction(|tx| {
        if let Some(entry) = BodyMetricsRepository::select_by_id_in(tx, date as i64)? {
            entry.delete_by_id_in(tx)?;
        }

        Ok(())
    });

    let lang = settings.read().unwrap().language;
    match res {
        Ok(_) => {
            info!("Measures deleted succesfully");
            show_notification(NotificationDefinition {
                title: translate("ok_delete_body_entry", lang),
                body: "".to_string(),
                kind: NotificationKind::Temporal,
            });
            Ok(())
        }
        Err(e) => Err(report_error(
            e,
            lang,
            "error_deleting_body_measures",
            "Error deleting measures",
        )),
    }
}
