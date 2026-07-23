use tauri_plugin_log::log::debug;

use super::types::value::Value;

pub mod insert;
pub mod select;
pub mod update;

pub trait QueryBuilder<T> {
    fn new() -> Self;

    fn log_query_start(sql: &str, params: &[Value]) {
        debug!(
            "Running statement {} with parameters [{}]",
            sql,
            params
                .iter()
                .map(|p| p.to_sql_str())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }

    fn log_query_ending(rows: usize) {
        debug!("Affected {} rows", rows);
    }
}
