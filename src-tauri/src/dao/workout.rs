use rusqlite_orm::Entity;
use serde::{Deserialize, Serialize};

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(name)]
pub struct Workout {
    pub name: String,
}
