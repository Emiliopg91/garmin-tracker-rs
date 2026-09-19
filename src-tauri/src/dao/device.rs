use rusqlite_orm::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Entity, Serialize, Deserialize)]
#[primary_key(serial)]
pub struct Device {
    pub serial: String,
    pub model: String,
    pub last_sync: Option<i64>,
}
