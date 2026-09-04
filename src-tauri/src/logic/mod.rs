pub mod app;
pub mod body_metrics;
pub mod devices;
pub mod exercises;
pub mod notifications;
pub mod sessions;
pub mod workouts;

use tauri_plugin_log::log::error;

use crate::{
    dto::notifications::{NotificationDefinition, NotificationKind},
    logic::notifications::show_notification,
    utils::translations::{Languages, translate},
};

/// Logs `e` (prefixed by `log_msg`), fires a persistent desktop notification titled by the
/// translation key `error_key`, and returns the stringified error for the command's
/// `Result<T, String>`.
pub fn report_error<E: std::fmt::Display>(
    e: E,
    lang: Languages,
    error_key: &str,
    log_msg: &str,
) -> String {
    error!("{}: {}", log_msg, e);
    show_notification(NotificationDefinition {
        title: translate(error_key, lang),
        body: e.to_string(),
        kind: NotificationKind::Persistant,
    });
    e.to_string()
}
