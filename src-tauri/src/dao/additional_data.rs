use rusqlite_orm_macros::Entity;

use crate::utils::constants::SEMICIRCLE_TO_DEGREES;

#[derive(Entity)]
#[entity("additional_data")]
#[primary_key(session)]
pub struct AdditionalData {
    pub session: i64,
    pub heart_rates: Option<Vec<u8>>,
    pub coordinates: Option<Vec<u8>>,
    pub speeds: Option<Vec<u8>>,
}

impl AdditionalData {
    pub fn get_coordinates_semicircle(&self) -> Option<Vec<(i32, i32)>> {
        if let Some(records) = &self.coordinates {
            Some(
                records
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|chunk| {
                        let lat = i32::from_be_bytes(chunk[0..4].try_into().unwrap());
                        let lon = i32::from_be_bytes(chunk[4..8].try_into().unwrap());
                        (lat, lon)
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
    pub fn get_coordinates(&self) -> Option<Vec<(f64, f64)>> {
        if let Some(semicircles) = self.get_coordinates_semicircle() {
            Some(
                semicircles
                    .into_iter()
                    .map(|(lat, long)| {
                        (
                            lat as f64 * SEMICIRCLE_TO_DEGREES,
                            long as f64 * SEMICIRCLE_TO_DEGREES,
                        )
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
    pub fn build_coordinates_blob(values: &[(i32, i32)]) -> Vec<u8> {
        let mut coords = Vec::new();

        for (latitude, longitude) in values {
            coords.extend_from_slice(&latitude.to_be_bytes());
            coords.extend_from_slice(&longitude.to_be_bytes());
        }

        coords
    }

    pub fn get_speeds(self) -> Option<Vec<f64>> {
        if let Some(records) = &self.speeds {
            Some(
                records
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|chunk| f64::from_be_bytes(*chunk))
                    .collect(),
            )
        } else {
            None
        }
    }

    pub fn build_speeds_blob(value: &[f64]) -> Vec<u8> {
        let mut records = Vec::new();

        value.to_vec().iter().for_each(|speed| {
            records.extend_from_slice(&(*speed).to_be_bytes());
        });

        records
    }
}
