use std::collections::HashMap;

use rusqlite_orm::{dao::Repository, database::DatabasePool};
use serde::Serialize;

use crate::{
    dao::{
        additional_data::{AdditionalData, AdditionalDataRepository},
        body_metric::{BodyMetric, BodyMetricRepository},
        device::{Device, DeviceRepository},
        serie::{Set, SetRepository},
        session::{Session, SessionRepository},
        settings::{Settings, SettingsRepository},
        workout::{Workout, WorkoutRepository},
    },
    utils::translations::{Languages, translate},
};

#[derive(Serialize)]
pub struct Export {
    body_metrics: Vec<BodyMetric>,
    devices: Vec<Device>,
    workouts: Vec<Workout>,
    sessions: Vec<SessionExport>,
    settings: Vec<Settings>,
}

impl Export {
    /// Loads every table from the database and assembles a full export snapshot.
    pub fn from_database(db: &DatabasePool, lang: Languages) -> rusqlite_orm::errors::Result<Self> {
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

            let mut series: HashMap<i64, Vec<Set>> = HashMap::new();
            SetRepository::select()
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

                    SessionExport::from((&session, add_data, serie, lang))
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

#[derive(Serialize)]
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
    pub series: Option<Vec<SetExport>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadences: Option<Vec<Option<u8>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powers: Option<Vec<Option<u16>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub respirations: Option<Vec<Option<f64>>>,
}

impl
    From<(
        &Session,
        Option<&AdditionalData>,
        Option<&Vec<Set>>,
        Languages,
    )> for SessionExport
{
    fn from(
        (session, additiona_data, sets, lang): (
            &Session,
            Option<&AdditionalData>,
            Option<&Vec<Set>>,
            Languages,
        ),
    ) -> Self {
        let mut heart_rates = None;
        let mut coordinates: Option<Vec<Option<(f64, f64)>>> = None;
        let mut speeds: Option<Vec<Option<f64>>> = None;
        let mut cadences = None;
        let mut powers = None;
        let mut respirations = None;

        if let Some(add_data) = additiona_data {
            heart_rates = add_data.get_heart_rates();
            coordinates = add_data.get_coordinates_degrees();
            speeds = add_data.get_speeds();
            cadences = add_data.get_cadences();
            powers = add_data.get_powers();
            respirations = add_data.get_respirations();
        }

        let series = if let Some(srs) = sets
            && !srs.is_empty()
        {
            Some(
                srs.into_iter()
                    .map(|s| {
                        let mut se = SetExport::from(s);
                        se.ex_name =
                            translate(&format!("exercise_{}_{}", se.ex_cat, se.ex_id), lang);
                        se
                    })
                    .collect(),
            )
        } else {
            None
        };

        Self {
            date: session.date,
            workout: session.name.clone(),
            total_elapsed_time: session.total_elapsed_time,
            active_time: session.active_time,
            total_calories: session.total_calories,
            metabolic_calories: session.metabolic_calories,
            training_load: session.training_load,
            sport: session.sport,
            sub_sport: session.sub_sport,
            device: session.device.clone(),
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

#[derive(Serialize)]
pub struct SetExport {
    pub idx: u8,
    pub ex_name: String,
    pub ex_cat: u16,
    pub ex_id: u16,
    pub reps: u16,
    pub weight: f64,
    pub pr: bool,
}

impl From<&Set> for SetExport {
    fn from(value: &Set) -> Self {
        Self {
            idx: value.idx,
            ex_cat: value.ex_cat,
            ex_id: value.ex_id,
            ex_name: String::new(),
            reps: value.reps,
            weight: value.weight,
            pr: value.pr,
        }
    }
}
