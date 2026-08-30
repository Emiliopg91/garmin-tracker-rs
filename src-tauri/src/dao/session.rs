use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

use crate::dao::{
    additional_data::{self, AdditionalData},
    device::{self, Device},
    serie,
};

use super::serie::Serie;

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(date)]
#[index("workout", (workout))]
pub struct Session {
    pub date: i64,
    pub workout: String,
    pub total_elapsed_time: f64,
    pub active_time: f64,
    pub total_calories: u16,
    pub metabolic_calories: u16,
    pub training_load: f64,
    pub sport: String,
    pub device: Option<String>,

    #[relationship((date, serie::entity::columns::SESSION))]
    pub series: Vec<Serie>,

    #[relationship((device, device::entity::columns::SERIAL))]
    pub device_obj: Option<Device>,

    #[relationship((date, additional_data::entity::columns::SESSION))]
    pub additional_data: Option<AdditionalData>,
}
