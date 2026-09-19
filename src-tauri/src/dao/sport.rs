use rusqlite_orm::Entity;
use serde::{Deserialize, Serialize};

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(id)]
pub struct Sport {
    pub id: u8,
}
