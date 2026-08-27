use rusqlite_orm::{
    dao::Repository,
    database::{Database, errors::DatabaseError},
};
use serde::{Deserialize, Serialize};

use crate::dao::{
    body_metrics::{BodyMetrics, BodyMetricsRepository},
    device::{self, Device, DeviceRepository},
    exercise::{Exercise, ExerciseRepository},
    gps_coordinates::{GpsCoordinates, GpsCoordinatesRepository},
    heart_rate::{HeartRate, HeartRateRepository},
    serie::{Serie, SerieRepository},
    session::{Session, SessionRepository},
    settings::{Settings, SettingsRepository},
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
    pub fn from_database() -> rusqlite_orm::database::errors::Result<Self> {
        Database::run_in_connection(|conn| {
            let body_metrics = BodyMetricsRepository::select().fetch_in(conn)?;
            let exercises = ExerciseRepository::select().fetch_in(conn)?;
            let devices = DeviceRepository::select().fetch_in(conn)?;
            let settings = SettingsRepository::select().fetch_in(conn)?;
            let mut sessions = SessionRepository::select().fetch_in(conn)?;
            for session in &mut sessions {
                session.fetch_device_obj_relationship_in_conn(conn)?;
                session.fetch_gps_coordinates_relationship_in_conn(conn)?;
                session.fetch_heart_rates_relationship_in_conn(conn)?;
                session.fetch_series_relationship_in_conn(conn)?;
            }
            let sessions = sessions
                .into_iter()
                .map(SessionExport::from)
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
    pub device: Option<String>,
    pub gps_coordinates: Option<Vec<(f64, f64)>>,
    pub heart_rates: Option<Vec<u8>>,
    pub series: Vec<Serie>,
}

impl From<Session> for SessionExport {
    fn from(session: Session) -> Self {
        let mut heart_rates = None;
        if let Some(hr) = session.heart_rates {
            heart_rates = Some(hr.records);
        }
        let mut gps_coordinates: Option<Vec<(f64, f64)>> = None;
        if let Some(gps) = session.gps_coordinates {
            gps_coordinates = Some((&gps).into());
        }
        Self {
            date: session.date,
            workout: session.workout,
            total_elapsed_time: session.total_elapsed_time,
            active_time: session.active_time,
            total_calories: session.total_calories,
            metabolic_calories: session.metabolic_calories,
            training_load: session.training_load,
            sport: session.sport,
            device: session.device,
            series: session.series,
            heart_rates,
            gps_coordinates,
        }
    }
}

impl From<SessionExport> for Session {
    fn from(session: SessionExport) -> Self {
        let mut heart_rates = None;
        if let Some(hr) = session.heart_rates {
            heart_rates = Some(HeartRate {
                session: session.date,
                records: hr,
            });
        }
        let mut gps_coordinates = None;
        if let Some(gps) = session.gps_coordinates {
            let mut obj = GpsCoordinates::from(gps.as_slice());
            obj.session = session.date;
            gps_coordinates = Some(obj);
        }
        Self {
            date: session.date,
            workout: session.workout,
            total_elapsed_time: session.total_elapsed_time,
            active_time: session.active_time,
            total_calories: session.total_calories,
            metabolic_calories: session.metabolic_calories,
            training_load: session.training_load,
            sport: session.sport,
            device: session.device,
            series: session.series,
            heart_rates,
            gps_coordinates,
            device_obj: None,
        }
    }
}
