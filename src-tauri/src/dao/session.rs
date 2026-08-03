use indexmap::IndexMap;
use rusqlite_orm::dao::{Repository, helpers::types::order_by::OrderBy};
use rusqlite_orm_macros::Entity;

use crate::dao::heart_rate::{self, HeartRate, HeartRateRepository};

use super::{exercise::Exercise, serie::Serie};

#[derive(Default, Entity, Clone)]
#[indexes((workout))]
pub struct Session {
    #[primary_key]
    pub date: i64,

    pub workout: String,

    pub total_elapsed_time: f64,
    pub active_time: f64,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub avg_heart_rate: u8,
    pub max_heart_rate: u8,

    pub training_load: f64,

    pub sub_sport: String,

    #[trasient]
    pub series: IndexMap<Exercise, Vec<Serie>>,

    #[trasient]
    pub heart_rates: Vec<HeartRate>,
}
impl Session {
    pub fn get_volume(&self) -> f64 {
        let mut volume = 0_f64;

        for (_, series) in &self.series {
            for serie in series {
                volume += (serie.reps as f64) * serie.weight
            }
        }

        volume
    }

    pub fn find_by_id(
        timestamp: i64,
        with_details: bool,
    ) -> rusqlite_orm::database::errors::Result<Option<Session>> {
        let opt_sess = SessionRepository::select_by_id(timestamp)?;

        Ok(match opt_sess {
            Some(mut session) => {
                if with_details {
                    session.series = if session.sub_sport == "strength_training" {
                        Serie::load_for_session(session.date)?
                    } else {
                        IndexMap::new()
                    };

                    session.heart_rates = HeartRateRepository::select_by_session(
                        session.date,
                        Some(&[OrderBy::Asc(heart_rate::entity::columns::IDX)]),
                    )?;
                }
                Some(session)
            }
            None => None,
        })
    }

    pub fn find_by_workout(workout: &str) -> rusqlite_orm::database::errors::Result<Vec<Session>> {
        let mut res = SessionRepository::select_by_workout(
            workout,
            Some(&[OrderBy::Desc(entity::columns::DATE)]),
        )?;

        for r in &mut res {
            r.series = Serie::load_for_session(r.date)?;
        }

        Ok(res)
    }

    pub fn load_from_db(with_series: bool) -> rusqlite_orm::database::errors::Result<Vec<Session>> {
        let mut res = SessionRepository::select()
            .order_by(OrderBy::Desc(entity::columns::DATE))
            .fetch()?;

        if with_series {
            for r in &mut res {
                r.series = if r.sub_sport == "strength_training" {
                    Serie::load_for_session(r.date)?
                } else {
                    IndexMap::new()
                };
            }
        }

        Ok(res)
    }
}
