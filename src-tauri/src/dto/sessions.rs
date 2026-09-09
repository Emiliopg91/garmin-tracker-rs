use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::dao::{exercise::Exercise, lap::Lap, serie::Serie, session::Session};

#[derive(Serialize, Default)]
pub struct SessionListItem {
    pub name: String,
    pub timestamp: i32,
    pub active_calories: u16,
    pub training_load: u16,
    pub sport: u8,
    pub sub_sport: u8,
    pub has_record: bool,
}

impl From<&Session> for SessionListItem {
    fn from(value: &Session) -> Self {
        Self {
            name: value.name.clone(),
            timestamp: value.date as i32,
            active_calories: value.total_calories - value.metabolic_calories,
            training_load: value.training_load,
            sport: value.sport,
            sub_sport: value.sub_sport,
            has_record: false,
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
    pub ex_cat: u16,
    pub ex_id: u16,
    pub idx: u8,
    pub reps: u16,
    pub weight: f64,
    pub pr: bool,
}

impl From<&Serie> for SessionSerie {
    fn from(value: &Serie) -> Self {
        Self {
            ex_cat: value.ex_cat,
            ex_id: value.ex_id,
            idx: value.idx,
            reps: value.reps,
            weight: value.weight,
            pr: value.pr,
        }
    }
}

#[derive(Serialize)]
pub struct SessionLap {
    idx: i32,
    start_latitude: Option<i32>,
    start_longitude: Option<i32>,
}

impl From<&Lap> for SessionLap {
    fn from(value: &Lap) -> Self {
        Self {
            idx: value.idx,
            start_latitude: value.start_latitude,
            start_longitude: value.start_longitude,
        }
    }
}

#[derive(Serialize)]
pub struct SessionDetails {
    pub name: String,

    pub timestamp: i32,

    pub total_elapsed_time: u32,
    pub active_time: u32,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub training_load: u16,
    pub sport: u8,
    pub sub_sport: u8,

    pub series: Vec<SessionSerie>,
    pub heart_rates: Vec<Option<u8>>,
    pub coordinates: Vec<Option<(i32, i32)>>,
    pub laps: Vec<SessionLap>,
    pub speeds: Vec<Option<f64>>,

    pub device: Option<String>,
}

impl From<(&Session, &[Exercise], &[Serie], &[Lap])> for SessionDetails {
    fn from(value: (&Session, &[Exercise], &[Serie], &[Lap])) -> Self {
        let device = value
            .0
            .device_obj
            .as_ref()
            .map(|dev| format!("Garmin {}", dev.model));

        let mut heart_rates = Vec::new();
        let mut gps_coordinates = Vec::new();
        let mut speeds = Vec::new();

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

        let series = value.2.iter().map(SessionSerie::from).collect();
        let laps = value.3.iter().map(SessionLap::from).collect();

        Self {
            name: value.0.name.clone(),
            timestamp: value.0.date as i32,
            total_elapsed_time: value.0.total_elapsed_time,
            active_time: value.0.active_time,
            metabolic_calories: value.0.metabolic_calories,
            total_calories: value.0.total_calories,
            training_load: value.0.training_load,
            sport: value.0.sport,
            sub_sport: value.0.sub_sport,
            series,
            heart_rates,
            device,
            laps,
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
