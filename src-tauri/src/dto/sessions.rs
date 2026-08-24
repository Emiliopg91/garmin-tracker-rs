use std::{collections::HashMap, hash::Hash};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::dao::{exercise::Exercise, serie::Serie, session::Session};

#[derive(Serialize, Default)]
pub struct SessionListItem {
    pub name: String,
    pub timestamp: i32,
    pub active_calories: u16,
    pub training_load: u16,
    pub sport: String,
}

impl From<&Session> for SessionListItem {
    fn from(value: &Session) -> Self {
        Self {
            name: value.workout.clone(),
            timestamp: value.date as i32,
            active_calories: value.total_calories - value.metabolic_calories,
            training_load: value.training_load.round() as u16,
            sport: value.sport.clone(),
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
    pub ex_cat: String,
    pub ex_id: u16,
    pub idx: u8,
    pub reps: u16,
    pub weight: f64,
}

impl From<&Serie> for SessionSerie {
    fn from(value: &Serie) -> Self {
        Self {
            ex_cat: value.ex_cat.clone(),
            ex_id: value.ex_id,
            idx: value.idx,
            reps: value.reps,
            weight: value.weight,
        }
    }
}

#[derive(Serialize)]
pub struct SessionDetails {
    pub name: String,

    pub timestamp: i32,

    pub total_elapsed_time: i32,
    pub active_time: i32,
    pub zones_times: Vec<i32>,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub training_load: u16,
    pub sport: String,

    pub exercises: Vec<String>,
    pub series: HashMap<String, Vec<SessionSerie>>,
    pub heart_rates: Vec<u8>,
    pub gps_coordinates: Vec<(f64, f64)>,

    pub device: Option<String>,
}

impl From<(&Session, &IndexMap<Exercise, Vec<Serie>>)> for SessionDetails {
    fn from(value: (&Session, &IndexMap<Exercise, Vec<Serie>>)) -> Self {
        let mut exercises = Vec::new();
        let mut series_d = HashMap::<String, Vec<SessionSerie>>::new();
        let mut name = value.0.workout.clone();

        for (exercise, series) in value.1 {
            if !exercises.contains(&exercise.name) {
                exercises.push(exercise.name.clone())
            }
            let entry = series_d.entry(exercise.name.clone()).or_default();
            for serie in series {
                entry.push(SessionSerie::from(serie));
            }
        }

        let device = value
            .0
            .device_obj
            .as_ref()
            .map(|dev| format!("Garmin {}", dev.model));

        let mut heart_rates = Vec::new();
        let mut zones_times = Vec::new();
        if let Some(hr_dao) = value.0.heart_rates.clone()
            && !hr_dao.records.is_empty()
        {
            heart_rates = hr_dao.records.clone().into_iter().collect();
            zones_times = hr_dao.get_time_in_zones(value.0.total_elapsed_time);
        }

        let mut gps_coordinates = Vec::new();
        if let Some(coords) = value.0.gps_coordinates.clone() {
            gps_coordinates = coords.normalize();
            if let Some(location) = coords.location {
                eprintln!("Location found");
                name = location;
            }
        }

        Self {
            name,
            timestamp: value.0.date as i32,
            total_elapsed_time: value.0.total_elapsed_time.round() as i32,
            active_time: value.0.active_time.round() as i32,
            zones_times,
            metabolic_calories: value.0.metabolic_calories,
            total_calories: value.0.total_calories,
            training_load: value.0.training_load.round() as u16,
            sport: value.0.sport.clone(),
            exercises,
            series: series_d,
            heart_rates,
            device,
            gps_coordinates,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SessionSeriesUpdate {
    pub timestamp: i32,
    pub series: Vec<SessionSerie>,
}
