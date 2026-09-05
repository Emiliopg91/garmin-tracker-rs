use std::collections::HashMap;

use serde::Serialize;

use crate::{dao::exercise::Exercise, dto::sessions::SessionSerie};

#[derive(Serialize)]
pub struct ExerciseListItem {
    pub category: u16,
    pub id: u16,
    pub reps: u16,
    pub weight: f64,
    pub rm: f64,
    pub date: i32,
}

impl From<&Exercise> for ExerciseListItem {
    fn from(value: &Exercise) -> Self {
        Self {
            category: value.category,
            id: value.id,
            reps: 0,
            weight: 0_f64,
            rm: 0_f64,
            date: 0,
        }
    }
}

#[derive(Serialize)]
pub struct ExerciseDetails {
    pub category: u16,
    pub id: u16,
    pub reps: u16,
    pub weight: f64,
    pub rm: f64,
    pub workouts: Vec<String>,
    pub series: HashMap<String, Vec<SessionSerie>>,
    pub pr_date: i32,
}

impl From<&Exercise> for ExerciseDetails {
    fn from(value: &Exercise) -> Self {
        Self {
            category: value.category,
            id: value.id,
            reps: 0,
            weight: 0_f64,
            rm: 0_f64,
            workouts: Vec::new(),
            series: HashMap::new(),
            pr_date: 0,
        }
    }
}
