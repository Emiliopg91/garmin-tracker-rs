use std::collections::HashMap;

use rusqlite_orm::{dao::Repository, database::DatabasePool};
use serde::{Deserialize, Serialize};

use crate::dao::{
    additional_data::{AdditionalData, AdditionalDataRepository},
    body_metric::{BodyMetric, BodyMetricRepository},
    device::{Device, DeviceRepository},
    serie::{Serie, SerieRepository},
    session::{Session, SessionRepository},
    settings::{Settings, SettingsRepository},
    workout::{Workout, WorkoutRepository},
};

#[derive(Serialize, Deserialize)]
pub struct Export {
    body_metrics: Vec<BodyMetric>,
    devices: Vec<Device>,
    workouts: Vec<Workout>,
    sessions: Vec<SessionExport>,
    settings: Vec<Settings>,
}

impl Export {
    /// Loads every table from the database and assembles a full export snapshot.
    pub fn from_database(db: &DatabasePool) -> rusqlite_orm::errors::Result<Self> {
        db.run_in_connection(|conn| {
            let body_metrics = BodyMetricRepository::select().fetch_in(conn)?;
            let workouts = WorkoutRepository::select().fetch_in(conn)?;
            let devices = DeviceRepository::select().fetch_in(conn)?;
            let settings = SettingsRepository::select().fetch_in(conn)?;
            let sessions = SessionRepository::select().fetch_in(conn)?;

            let mut additional_datas: HashMap<i64, AdditionalData> = HashMap::new();
            AdditionalDataRepository::select()
                .fetch_in(conn)?
                .into_iter()
                .for_each(|ad| {
                    additional_datas.insert(ad.session, ad);
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
                    let serie = series.get(&session.date);
                    let add_data = additional_datas.get(&session.date);

                    SessionExport::from((&session, add_data, serie))
                })
                .collect::<Vec<_>>();

            Ok(Self {
                body_metrics,
                devices,
                workouts,
                sessions,
                settings,
            })
        })
    }

    /// Serializes the export snapshot to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SessionExport {
    pub date: i64,
    pub workout: String,
    pub total_elapsed_time: u32,
    pub active_time: u32,
    pub total_calories: u16,
    pub metabolic_calories: u16,
    pub training_load: u16,
    pub sport: u8,
    pub sub_sport: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<Vec<Option<(f64, f64)>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speeds: Option<Vec<Option<f64>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heart_rates: Option<Vec<Option<u8>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<Vec<Serie>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadences: Option<Vec<Option<u8>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powers: Option<Vec<Option<u16>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub respirations: Option<Vec<Option<f64>>>,
}

impl From<(&Session, Option<&AdditionalData>, Option<&Vec<Serie>>)> for SessionExport {
    fn from(values: (&Session, Option<&AdditionalData>, Option<&Vec<Serie>>)) -> Self {
        let mut heart_rates = None;
        let mut coordinates: Option<Vec<Option<(f64, f64)>>> = None;
        let mut speeds: Option<Vec<Option<f64>>> = None;
        let mut series = None;
        let mut cadences = None;
        let mut powers = None;
        let mut respirations = None;

        if let Some(add_data) = values.1 {
            heart_rates = add_data.get_heart_rates();
            coordinates = add_data.get_coordinates_degrees();
            speeds = add_data.get_speeds();
            cadences = add_data.get_cadences();
            powers = add_data.get_powers();
            respirations = add_data.get_respirations();
        }

        if let Some(srs) = values.2
            && !srs.is_empty()
        {
            series = Some(srs.clone());
        }

        Self {
            date: values.0.date,
            workout: values.0.name.clone(),
            total_elapsed_time: values.0.total_elapsed_time,
            active_time: values.0.active_time,
            total_calories: values.0.total_calories,
            metabolic_calories: values.0.metabolic_calories,
            training_load: values.0.training_load,
            sport: values.0.sport,
            sub_sport: values.0.sub_sport,
            device: values.0.device.clone(),
            series,
            heart_rates,
            coordinates,
            speeds,
            cadences,
            powers,
            respirations,
        }
    }
}
