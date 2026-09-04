use std::hash::Hash;

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
    pub exercise: String,
    pub idx: u8,
    pub reps: u16,
    pub weight: f64,
}

impl From<(&Serie, &str)> for SessionSerie {
    fn from(value: (&Serie, &str)) -> Self {
        Self {
            ex_cat: value.0.ex_cat.clone(),
            ex_id: value.0.ex_id,
            exercise: value.1.to_string(),
            idx: value.0.idx,
            reps: value.0.reps,
            weight: value.0.weight,
        }
    }
}

#[derive(Serialize)]
pub struct SessionDetails {
    pub name: String,

    pub timestamp: i32,

    pub total_elapsed_time: i32,
    pub active_time: i32,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub training_load: u16,
    pub sport: String,

    pub series: Vec<SessionSerie>,
    pub heart_rates: Vec<Option<u8>>,
    pub coordinates: Vec<Option<(i32, i32)>>,
    pub speeds: Vec<Option<f64>>,

    pub device: Option<String>,
}

impl From<(&Session, &[Exercise], &[Serie])> for SessionDetails {
    fn from(value: (&Session, &[Exercise], &[Serie])) -> Self {
        let device = value
            .0
            .device_obj
            .as_ref()
            .map(|dev| format!("Garmin {}", dev.model));

        let mut heart_rates = Vec::new();
        let mut gps_coordinates: Vec<Option<(i32, i32)>> = Vec::new();
        let mut speeds: Vec<Option<f64>> = Vec::new();

        if let Some(add_data) = &value.0.additional_data {
            if let Some(hr_data) = add_data.get_heart_rates() {
                heart_rates = hr_data;
            }
            if let Some(coords) = add_data.get_coordinates_semicircle() {
                gps_coordinates = coords;
            }
            if let Some(spds) = add_data.get_speeds() {
                speeds = spds;
            }
        }

        let series = value
            .2
            .iter()
            .map(|s| {
                let exercise = value
                    .1
                    .iter()
                    .find(|e| e.id == s.ex_id && e.category == s.ex_cat)
                    .unwrap();

                SessionSerie::from((s, exercise.name.as_str()))
            })
            .collect::<Vec<_>>();

        Self {
            name: value.0.workout.clone(),
            timestamp: value.0.date as i32,
            total_elapsed_time: value.0.total_elapsed_time.round() as i32,
            active_time: value.0.active_time.round() as i32,
            metabolic_calories: value.0.metabolic_calories,
            total_calories: value.0.total_calories,
            training_load: value.0.training_load.round() as u16,
            sport: value.0.sport.clone(),
            series,
            heart_rates,
            device,
            coordinates: gps_coordinates,
            speeds,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct SessionSeriesUpdate {
    pub timestamp: i32,
    pub series: Vec<SessionSerie>,
}

#[derive(Serialize, Clone)]
pub struct SessionLocation {
    pub session: i32,
    pub location: String,
}
