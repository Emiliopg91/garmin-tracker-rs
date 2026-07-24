pub mod delete;
pub mod insert;
pub mod select;
pub mod update;

use tauri_plugin_log::log::debug;

use super::types::value::Value;

pub trait QueryBuilder<T> {
    fn new() -> Self;

    fn log_query_start(sql: &str, params: &[Value]) {
        let mut sentence = sql.to_string();
        for param in params {
            let mut sql_str = param.to_sql_str();
            sql_str = match param {
                Value::Text(_) => format!("'{}'", sql_str),
                _ => sql_str,
            };
            sentence = sentence.replacen("?", &sql_str, 1);
        }

        debug!("Running statement \"{}\"", sentence,);
    }

    fn log_query_ending(rows: usize, is_fetch: bool) {
        let s = if rows > 1 { "s" } else { "" };
        if is_fetch {
            debug!("Fetched {} row{}", rows, s);
        } else {
            debug!("Affected {} row{}", rows, s);
        }
    }
}
