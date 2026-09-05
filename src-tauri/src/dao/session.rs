use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

use crate::dao::{
    additional_data::{self, AdditionalData},
    device::{self, Device},
    serie,
    sub_sport::{self, SubSport},
    workout::{self, Workout},
};

use super::serie::Serie;

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(date)]
#[index("name", (name))]
pub struct Session {
    pub date: i64,
    pub name: String,
    pub total_elapsed_time: u32,
    pub active_time: u32,
    pub total_calories: u16,
    pub metabolic_calories: u16,
    pub training_load: u16,
    pub device: Option<String>,
    pub workout: Option<String>,
    pub sport: Option<u8>,
    pub sub_sport: Option<u8>,

    #[relationship((date, serie::entity::columns::SESSION))]
    pub series: Vec<Serie>,

    #[relationship((device, device::entity::columns::SERIAL))]
    pub device_obj: Option<Device>,

    #[relationship((date, additional_data::entity::columns::SESSION))]
    pub additional_data: Option<AdditionalData>,

    #[relationship((sport, sub_sport::entity::columns::SPORT),(sub_sport, sub_sport::entity::columns::ID))]
    pub sub_sport_obj: Option<SubSport>,

    #[relationship((workout, workout::entity::columns::NAME))]
    pub workout_obj: Option<Workout>,
}
