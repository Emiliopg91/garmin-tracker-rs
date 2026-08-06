use rusqlite_orm_macros::Entity;

use crate::dao::{
    device::{self, Device},
    serie,
};

use super::serie::Serie;

#[derive(Entity, Clone)]
#[primary_key(date)]
#[index("workout", (workout))]
pub struct Session {
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
    pub device: Option<String>,
    pub heart_rates: Vec<u8>,

    #[relationship((date, serie::entity::columns::SESSION))]
    pub series: Vec<Serie>,

    #[relationship((device, device::entity::columns::SERIAL))]
    pub device_obj: Option<Device>,
}
