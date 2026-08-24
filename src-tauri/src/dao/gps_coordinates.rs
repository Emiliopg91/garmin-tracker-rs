use rusqlite_orm_macros::Entity;

use crate::utils::constants;

#[derive(Clone, Entity)]
#[entity("gps_coordinates")]
#[primary_key(session)]
pub struct GpsCoordinates {
    pub session: i64,
    pub records: Vec<u8>,
    pub location: Option<String>,
}

impl GpsCoordinates {
    pub fn normalize(&self) -> Vec<(f64, f64)> {
        let mut gps_coordinates = Vec::new();
        let mut idx = 0;
        while idx < self.records.len() {
            let lat = i32::from_be_bytes(self.records[idx..idx + 4].try_into().unwrap()) as f64
                * constants::SEMICIRCLE_TO_DEGREES;

            let lon = i32::from_be_bytes(self.records[idx + 4..idx + 8].try_into().unwrap()) as f64
                * constants::SEMICIRCLE_TO_DEGREES;

            gps_coordinates.push((lat, lon));

            idx += 8;
        }
        gps_coordinates
    }
}
