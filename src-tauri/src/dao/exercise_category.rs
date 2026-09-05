use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Entity, Serialize, Deserialize)]
#[entity("exercise_category")]
#[primary_key(id)]
pub struct ExerciseCategory {
    pub id: u16,
}
