use std::{collections::HashMap, hash::Hash};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    dao::{exercise::Exercise, serie::Serie, session::Session},
    utils::date_time_utils::DateTimeUtils,
};

#[derive(Serialize, Default)]
pub struct SessionListItem {
    pub name: String,
    pub date: String,
    pub timestamp: i64,
    pub volume: f64,
    pub active_calories: u16,
    pub training_load: u16,
    pub sport: String,
}

impl From<&Session> for SessionListItem {
    fn from(value: &Session) -> Self {
        Self {
            name: value.workout.clone(),
            date: DateTimeUtils::format_time_date(value.date),
            timestamp: value.date,
            active_calories: value.total_calories - value.metabolic_calories,
            volume: value.volume,
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

    pub date: String,
    pub timestamp: i64,

    pub total_elapsed_time: String,
    pub active_time: String,
    pub zones_times: Vec<String>,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub training_load: u16,

    pub avg_heart_rate: u8,
    pub max_heart_rate: u8,
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

            let time_fraction = value.0.total_elapsed_time / (heart_rates.len() as f64);
            let mut num_zones_times = Vec::new();
            let max_hr = 189_u8.max(*heart_rates.iter().max().unwrap()) as f64;
            for zone in 0..5 {
                let entries = heart_rates
                    .iter()
                    .filter(|e| {
                        let rate = (**e as f64) / max_hr;
                        let local_zone = if rate < 0.6 {
                            0
                        } else {
                            if rate < 0.7 {
                                1
                            } else {
                                if rate < 0.8 {
                                    2
                                } else {
                                    if rate < 0.9 { 3 } else { 4 }
                                }
                            }
                        };
                        local_zone == zone
                    })
                    .count();
                let time = time_fraction * (entries as f64);
                num_zones_times.push(time);
            }

            let mut int_times = num_zones_times
                .iter()
                .map(|s| s.round() as i64)
                .collect::<Vec<_>>();
            let acc: i64 = int_times.iter().sum();
            let total_secs = value.0.total_elapsed_time.round() as i64;
            let diff = total_secs - acc;
            int_times[0] += diff;

            zones_times = int_times
                .into_iter()
                .map(|v| {
                    if v <= 0 {
                        "0:00".to_string()
                    } else {
                        let mut r = DateTimeUtils::format_duration(v as u64);
                        if !r.contains(':') {
                            r = format!("0:{}", r);
                        }
                        r
                    }
                })
                .collect();
        }

        let mut gps_coordinates = Vec::new();
        if let Some(coords) = value.0.gps_coordinates.clone() {
            const SEMICIRCLE_TO_DEGREES: f64 = 180.0 / (2_i64.pow(31) as f64);

            fn semicircles_to_degrees(semicircles: i32) -> f64 {
                semicircles as f64 * SEMICIRCLE_TO_DEGREES
            }

            let mut idx = 0;
            while idx < coords.records.len() {
                let lat = semicircles_to_degrees(i32::from_be_bytes(
                    coords.records[idx..idx + 4].try_into().unwrap(),
                ));
                let lon = semicircles_to_degrees(i32::from_be_bytes(
                    coords.records[idx + 4..idx + 8].try_into().unwrap(),
                ));

                gps_coordinates.push((lat, lon));

                idx += 8;
            }
        }

        Self {
            name: value.0.workout.clone(),
            date: DateTimeUtils::format_time_date(value.0.date),
            timestamp: value.0.date,
            active_time: DateTimeUtils::format_duration(value.0.active_time as u64),
            zones_times,
            avg_heart_rate: value.0.avg_heart_rate,
            max_heart_rate: value.0.max_heart_rate,
            metabolic_calories: value.0.metabolic_calories,
            total_calories: value.0.total_calories,
            total_elapsed_time: DateTimeUtils::format_duration(value.0.total_elapsed_time as u64),
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
    pub timestamp: i64,
    pub series: Vec<SessionSerie>,
}
