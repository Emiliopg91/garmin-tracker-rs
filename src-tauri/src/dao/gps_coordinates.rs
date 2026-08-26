use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

use crate::utils::constants::SEMICIRCLE_TO_DEGREES;

#[derive(Clone, Entity, Serialize, Deserialize)]
#[entity("gps_coordinates")]
#[primary_key(session)]
pub struct GpsCoordinates {
    pub session: i64,
    pub records: Vec<u8>,
}

impl From<&GpsCoordinates> for Vec<(i32, i32)> {
    fn from(value: &GpsCoordinates) -> Self {
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

impl From<&GpsCoordinates> for Vec<(f64, f64)> {
    fn from(value: &GpsCoordinates) -> Self {
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

impl From<&[(f64, f64)]> for GpsCoordinates {
    fn from(value: &[(f64, f64)]) -> Self {
        let mut records = Vec::new();

        value.to_vec().iter().for_each(|(lat, long)| {
            records.extend_from_slice(&(*lat).to_be_bytes());
            records.extend_from_slice(&(*long).to_be_bytes());
        });

        GpsCoordinates {
            session: 0_i64,
            records,
        }
    }
}
