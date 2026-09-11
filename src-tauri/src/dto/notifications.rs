use std::time::Duration;

use serde::Deserialize;

#[derive(Deserialize)]
pub enum NotificationKind {
    Temporal,
    Custom(Duration),
    Persistant,
}

impl NotificationKind {
    /// Display timeout in milliseconds for this notification kind (`0` means persistent, no auto-dismiss).
    pub fn get_timeout(&self) -> u128 {
        match self {
            Self::Temporal => 5000,
            Self::Custom(v) => v.as_millis(),
            Self::Persistant => 0,
        }
    }
}

#[derive(Deserialize)]
pub struct NotificationDefinition {
    pub title: String,
    pub body: String,
    pub kind: NotificationKind,
}
