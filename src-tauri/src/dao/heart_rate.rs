use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Entity, Serialize, Deserialize)]
#[entity("heart_rate")]
#[primary_key(session)]
pub struct HeartRate {
    pub session: i64,
    pub records: Vec<u8>,
}
