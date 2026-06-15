use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::garmin::database::dao::{serie::Serie, session::Session};

#[derive(Serialize, Type)]
pub struct WorkoutListItem {
    pub name: String,
    pub date: String,
    #[specta(type = String)]
    pub timestamp: i64,
}

impl From<&Session> for WorkoutListItem {
    fn from(value: &Session) -> Self {
        Self {
            name: value.workout.clone(),
            date: value.format_date(),
            timestamp: value.timestamp.timestamp(),
        }
    }
}
impl PartialEq for WorkoutListItem {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}
impl Eq for WorkoutListItem {}
impl Hash for WorkoutListItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.timestamp.hash(state);
    }
}

#[derive(Serialize, Deserialize, Type)]
pub struct WorkoutSerie {
    pub idx: u8,
    pub reps: u16,
    pub weight: f64,
}

impl From<&Serie> for WorkoutSerie {
    fn from(value: &Serie) -> Self {
        Self {
            idx: value.idx,
            reps: value.reps,
            weight: value.weight,
        }
    }
}

#[derive(Serialize, Type)]
pub struct WorkoutDetails {
    pub name: String,

    pub date: String,
    #[specta(type = String)]
    pub timestamp: i64,

    pub total_elapsed_time: String,
    pub active_time: String,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub avg_heart_rate: u8,
    pub max_heart_rate: u8,
    pub exercises: Vec<String>,
    pub series: HashMap<String, Vec<WorkoutSerie>>,
}

impl From<&Session> for WorkoutDetails {
    fn from(value: &Session) -> Self {
        Self {
            name: value.workout.clone(),
            date: value.format_date(),
            timestamp: value.timestamp.timestamp(),
            active_time: value.format_active_time(),
            avg_heart_rate: value.avg_heart_rate,
            max_heart_rate: value.max_heart_rate,
            metabolic_calories: value.metabolic_calories,
            total_calories: value.total_calories,
            total_elapsed_time: value.format_total_time(),
            exercises: Vec::new(),
            series: HashMap::new(),
        }
    }
}

#[derive(Deserialize, Type)]
pub struct WorkoutSeriesUpdate {
    #[specta(type = String)]
    pub timestamp: i64,
    pub series: Vec<WorkoutSerie>,
}
