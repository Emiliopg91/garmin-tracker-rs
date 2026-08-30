use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

use crate::utils::constants::SEMICIRCLE_TO_DEGREES;

pub const POSITION_INVALID: i32 = 0x7FFFFFFF; // 2147483647

#[derive(Entity, Clone, Deserialize, Serialize)]
#[entity("additional_data")]
#[primary_key(session)]
pub struct AdditionalData {
    pub session: i64,
    pub heart_rates: Option<Vec<u8>>,
    pub coordinates: Option<Vec<u8>>,
    pub speeds: Option<Vec<u8>>,
}

impl AdditionalData {
    /// Unpacks the raw byte blob into (lat, lon) pairs in semicircles.
    pub fn get_coordinates_semicircle(&self) -> Option<Vec<Option<(i32, i32)>>> {
        self.coordinates.as_ref().map(|records| {
            records
                .as_chunks::<8>()
                .0
                .iter()
                .map(|chunk| {
                    let lat = i32::from_be_bytes(chunk[0..4].try_into().unwrap());
                    let lon = i32::from_be_bytes(chunk[4..8].try_into().unwrap());
                    if lat != POSITION_INVALID && lon != POSITION_INVALID {
                        Some((lat, lon))
                    } else {
                        None
                    }
                })
                .collect()
        })
    }
    /// Unpacks the raw byte blob into (lat, lon) pairs converted to degrees.
    pub fn get_coordinates_degrees(&self) -> Option<Vec<Option<(f64, f64)>>> {
        self.get_coordinates_semicircle().map(|semicircles| {
            semicircles
                .into_iter()
                .map(|p| {
                    p.map(|p| {
                        (
                            p.0 as f64 * SEMICIRCLE_TO_DEGREES,
                            p.1 as f64 * SEMICIRCLE_TO_DEGREES,
                        )
                    })
                })
                .collect()
        })
    }
    /// Build blob from semicircles coordinates Vec
    pub fn build_coordinates_blob(values: &[(i32, i32)]) -> Vec<u8> {
        let mut coords = Vec::new();

        for (latitude, longitude) in values {
            coords.extend_from_slice(&latitude.to_be_bytes());
            coords.extend_from_slice(&longitude.to_be_bytes());
        }

        coords
    }

    /// Unpacks the speeds Blob into Vec
    pub fn get_speeds(&self) -> Option<Vec<f64>> {
        self.speeds.as_ref().map(|records| {
            records
                .as_chunks::<8>()
                .0
                .iter()
                .map(|chunk| f64::from_be_bytes(*chunk))
                .collect()
        })
    }
    /// Build blob from speeds Vec
    pub fn build_speeds_blob(value: &[f64]) -> Vec<u8> {
        let mut records = Vec::new();

        value.to_vec().iter().for_each(|speed| {
            records.extend_from_slice(&(*speed).to_be_bytes());
        });

        records
    }
}
