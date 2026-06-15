use indexmap::IndexMap;
use serde::Serialize;

use crate::{garmin::database::dao::exercise::Exercise, models::workouts::WorkoutSerie};

#[derive(Serialize)]
pub struct ExerciseListItem {
    pub category: String,
    pub id: u16,
    pub name: String,
    pub reps: u16,
    pub weight: f64,
    pub rm: f64,
}

impl From<&Exercise> for ExerciseListItem {
    fn from(value: &Exercise) -> Self {
        Self {
            category: value.category.clone(),
            id: value.id,
            name: value.name.clone(),
            reps: 0,
            weight: 0_f64,
            rm: 0_f64,
        }
    }
}

#[derive(Serialize)]
pub struct ExerciseDetails {
    pub category: String,
    pub id: u16,
    pub name: String,
    pub reps: u16,
    pub weight: f64,
    pub rm: f64,
    pub workouts: IndexMap<String, Vec<WorkoutSerie>>,
}

impl From<&Exercise> for ExerciseDetails {
    fn from(value: &Exercise) -> Self {
        Self {
            category: value.category.clone(),
            id: value.id,
            name: value.name.clone(),
            reps: 0,
            weight: 0_f64,
            rm: 0_f64,
            workouts: IndexMap::new(),
        }
    }
}
