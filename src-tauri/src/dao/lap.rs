use rusqlite_orm::Entity;
use serde::{Deserialize, Serialize};

use crate::dao::additional_data::AdditionalData;

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(session, idx)]
#[index("session", (session))]
pub struct Lap {
    pub session: i64,
    pub idx: i32,
    pub start_latitude: Option<i32>,
    pub start_longitude: Option<i32>,
}

impl Lap {
    pub fn get_coordinates_degrees(&self) -> Option<(f64, f64)> {
        if let Some(lat) = self.start_latitude
            && let Some(lon) = self.start_longitude
        {
            Some((
                lat as f64 * AdditionalData::SEMICIRCLE_TO_DEGREES,
                lon as f64 * AdditionalData::SEMICIRCLE_TO_DEGREES,
            ))
        } else {
            None
        }
    }
}
