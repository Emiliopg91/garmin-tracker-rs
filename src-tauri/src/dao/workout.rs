use rusqlite_orm::Entity;
use serde::{Deserialize, Serialize};

use crate::dao::workout_step::{self, WorkoutStep};

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(name)]
pub struct Workout {
    pub name: String,
    pub enabled: bool,

    #[relationship((name, workout_step::entity::columns::WORKOUT))]
    pub steps: Vec<WorkoutStep>,
}

impl Workout {
    pub fn get_workout_file_name(&self) -> String {
        format!("{}.fit", self.name.replace(" ", "_"))
    }
}
