use rusqlite_orm_macros::Entity;

use crate::dao::{
    heart_rate::{self, HeartRate},
    serie,
};

use super::serie::Serie;

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

    #[relationship((date, serie::entity::columns::SESSION))]
    pub series: Vec<Serie>,

    #[relationship((date, heart_rate::entity::columns::SESSION))]
    pub heart_rates: Vec<HeartRate>,
}

impl Session {
    pub fn get_volume(&self) -> f64 {
        let mut volume = 0_f64;

        for serie in &self.series {
            volume += (serie.reps as f64) * serie.weight
        }

        volume
    }
}
