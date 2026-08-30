use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

use crate::utils::constants::SEMICIRCLE_TO_DEGREES;

#[derive(Clone, Entity, Serialize, Deserialize)]
#[primary_key(session)]
pub struct Coordinates {
    pub session: i64,
    pub records: Vec<u8>,
}

/// Unpacks the raw byte blob into (lat, lon) pairs in semicircles.
impl From<&Coordinates> for Vec<(i32, i32)> {
    fn from(value: &Coordinates) -> Self {
        value
            .records
            .as_chunks::<8>()
            .0
            .iter()
            .map(|chunk| {
                let lat = i32::from_be_bytes(chunk[0..4].try_into().unwrap());
                let lon = i32::from_be_bytes(chunk[4..8].try_into().unwrap());
                (lat, lon)
            })
            .collect()
    }
}

/// Unpacks the raw byte blob into (lat, lon) pairs converted to degrees.
impl From<&Coordinates> for Vec<(f64, f64)> {
    fn from(value: &Coordinates) -> Self {
        let semicircles: Vec<(i32, i32)> = value.into();
        semicircles
            .into_iter()
            .map(|(lat, long)| {
                (
                    lat as f64 * SEMICIRCLE_TO_DEGREES,
                    long as f64 * SEMICIRCLE_TO_DEGREES,
                )
            })
            .collect()
    }
}
