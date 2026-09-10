use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(session, idx)]
#[index("session", (session))]
pub struct Lap {
    #[autoincrement]
    pub session: i64,
    pub idx: i32,
    pub start_latitude: Option<i32>,
    pub start_longitude: Option<i32>,
}
