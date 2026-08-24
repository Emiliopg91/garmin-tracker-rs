use rusqlite_orm_macros::Entity;

use crate::utils::constants;

#[derive(Clone, Entity)]
#[entity("gps_coordinates")]
#[primary_key(session)]
pub struct GpsCoordinates {
    pub session: i64,
    pub records: Vec<u8>,
}

impl GpsCoordinates {
    pub fn normalize(&self) -> Vec<(f64, f64)> {
        self.records
            .as_chunks::<8>()
            .0
            .iter()
            .map(|chunk| {
                let lat = i32::from_be_bytes(chunk[0..4].try_into().unwrap()) as f64
                    * constants::SEMICIRCLE_TO_DEGREES;
                let lon = i32::from_be_bytes(chunk[4..8].try_into().unwrap()) as f64
                    * constants::SEMICIRCLE_TO_DEGREES;
                (lat, lon)
            })
            .collect()
    }
}
