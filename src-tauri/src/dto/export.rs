use std::collections::HashMap;

use rusqlite_orm::{dao::Repository, database::Database};
use serde::{Deserialize, Serialize};

use crate::dao::{
    body_metrics::{BodyMetrics, BodyMetricsRepository},
    coordinates::{Coordinates, CoordinatesRepository},
    device::{Device, DeviceRepository},
    exercise::{Exercise, ExerciseRepository},
    heart_rate::{HeartRate, HeartRateRepository},
    serie::{Serie, SerieRepository},
    session::{Session, SessionRepository},
    settings::{Settings, SettingsRepository},
    speeds::{Speeds, SpeedsRepository},
};

#[derive(Serialize, Deserialize)]
pub struct Export {
    body_metrics: Vec<BodyMetrics>,
    devices: Vec<Device>,
    exercises: Vec<Exercise>,
    sessions: Vec<SessionExport>,
    settings: Vec<Settings>,
}

impl Export {
    /// Loads every table from the database and assembles a full export snapshot.
    pub fn from_database() -> rusqlite_orm::database::errors::Result<Self> {
        Database::run_in_connection(|conn| {
            let body_metrics = BodyMetricsRepository::select().fetch_in(conn)?;
            let exercises = ExerciseRepository::select().fetch_in(conn)?;
            let devices = DeviceRepository::select().fetch_in(conn)?;
            let settings = SettingsRepository::select().fetch_in(conn)?;
            let sessions = SessionRepository::select().fetch_in(conn)?;

            let mut heart_rates: HashMap<i64, HeartRate> = HashMap::new();
            HeartRateRepository::select()
                .fetch_in(conn)?
                .into_iter()
                .for_each(|hr| {
                    heart_rates.insert(hr.session, hr);
                });

            let mut coordinates: HashMap<i64, Coordinates> = HashMap::new();
            CoordinatesRepository::select()
                .fetch_in(conn)?
                .into_iter()
                .for_each(|c| {
                    coordinates.insert(c.session, c);
                });

            let mut speeds: HashMap<i64, Speeds> = HashMap::new();
            SpeedsRepository::select()
                .fetch_in(conn)?
                .into_iter()
                .for_each(|s| {
                    speeds.insert(s.session, s);
                });

            let mut series: HashMap<i64, Vec<Serie>> = HashMap::new();
            SerieRepository::select()
                .fetch_in(conn)?
                .into_iter()
                .for_each(|s| {
                    let entry = series.entry(s.session).or_default();
                    entry.push(s);
                });

            let sessions = sessions
                .into_iter()
                .map(|session| {
                    let heart_rate = heart_rates.get(&session.date);
                    let coordinate = coordinates.get(&session.date);
                    let speed = speeds.get(&session.date);
                    let serie = series.get(&session.date);

                    SessionExport::from((&session, heart_rate, coordinate, speed, serie))
                })
                .collect::<Vec<_>>();

            Ok(Self {
                body_metrics,
                devices,
                exercises,
                sessions,
                settings,
            })
        })
    }

    /// Serializes the export snapshot to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SessionExport {
    pub date: i64,
    pub workout: String,
    pub total_elapsed_time: f64,
    pub active_time: f64,
    pub total_calories: u16,
    pub metabolic_calories: u16,
    pub training_load: f64,
    pub sport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<Vec<(f64, f64)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speeds: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heart_rates: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<Vec<Serie>>,
}

impl
    From<(
        &Session,
        Option<&HeartRate>,
        Option<&Coordinates>,
        Option<&Speeds>,
        Option<&Vec<Serie>>,
    )> for SessionExport
{
    fn from(
        values: (
            &Session,
            Option<&HeartRate>,
            Option<&Coordinates>,
            Option<&Speeds>,
            Option<&Vec<Serie>>,
        ),
    ) -> Self {
        let mut heart_rates = None;
        if let Some(hr) = values.1 {
            heart_rates = Some(hr.records.clone());
        }
        let mut coordinates: Option<Vec<(f64, f64)>> = None;
        if let Some(gps) = values.2 {
            coordinates = Some(gps.into());
        }
        let mut speeds: Option<Vec<f64>> = None;
        if let Some(spds) = values.3 {
            speeds = Some(spds.into());
        }
        let mut series = None;
        if let Some(srs) = values.4
            && !srs.is_empty()
        {
            series = Some(srs.clone());
        }

        Self {
            date: values.0.date,
            workout: values.0.workout.clone(),
            total_elapsed_time: values.0.total_elapsed_time,
            active_time: values.0.active_time,
            total_calories: values.0.total_calories,
            metabolic_calories: values.0.metabolic_calories,
            training_load: values.0.training_load,
            sport: values.0.sport.clone(),
            device: values.0.device.clone(),
            series,
            heart_rates,
            coordinates,
            speeds,
        }
    }
}
