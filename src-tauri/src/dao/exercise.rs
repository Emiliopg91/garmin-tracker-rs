use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Entity, Serialize, Deserialize)]
#[entity(hashable = true, comparable = true)]
#[primary_key(category, id)]
pub struct Exercise {
    pub category: String,
    pub id: u16,
    pub name: String,
}
