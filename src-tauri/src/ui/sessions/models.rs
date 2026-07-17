use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

use crate::garmin::database::dao::{serie::Serie, session::Session};

#[derive(Serialize, Default)]
pub struct SessionListItem {
    pub name: String,
    pub date: String,
    pub timestamp: i64,
    pub volume: f64,
    pub active_calories: u16,
    pub training_load: u16,
    pub sub_sport: String,
}

impl From<&Session> for SessionListItem {
    fn from(value: &Session) -> Self {
        Self {
            name: value.workout.clone(),
            date: value.format_date(),
            timestamp: value.timestamp.timestamp(),
            active_calories: value.total_calories - value.metabolic_calories,
            volume: value.get_volume(),
            training_load: value.training_load.round() as u16,
            sub_sport: value.sub_sport.clone(),
        }
    }
}
impl PartialEq for SessionListItem {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}
impl Eq for SessionListItem {}
impl Hash for SessionListItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.timestamp.hash(state);
    }
}

#[derive(Serialize, Deserialize)]
pub struct SessionSerie {
    pub idx: u8,
    pub reps: u16,
    pub weight: f64,
}

impl From<&Serie> for SessionSerie {
    fn from(value: &Serie) -> Self {
        Self {
            idx: value.idx,
            reps: value.reps,
            weight: value.weight,
        }
    }
}

#[derive(Serialize)]
pub struct SessionDetails {
    pub name: String,

    pub date: String,
    pub timestamp: i64,

    pub total_elapsed_time: String,
    pub active_time: String,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub training_load: u16,

    pub avg_heart_rate: u8,
    pub max_heart_rate: u8,
    pub sub_sport: String,

    pub exercises: Vec<String>,
    pub series: HashMap<String, Vec<SessionSerie>>,
}

impl From<&Session> for SessionDetails {
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
            training_load: value.training_load.round() as u16,
            sub_sport: value.sub_sport.clone(),
            exercises: Vec::new(),
            series: HashMap::new(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SessionSeriesUpdate {
    pub timestamp: i64,
    pub series: Vec<SessionSerie>,
}
