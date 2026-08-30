use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Entity, Serialize, Deserialize)]
#[primary_key(session)]
pub struct Speeds {
    pub session: i64,
    pub records: Vec<u8>,
}

/// Unpacks the raw byte blob into individual speed samples.
impl From<&Speeds> for Vec<f64> {
    fn from(value: &Speeds) -> Self {
        value
            .records
            .as_chunks::<8>()
            .0
            .iter()
            .map(|chunk| f64::from_be_bytes(*chunk))
            .collect()
    }
}

/// Packs speed samples into the raw byte blob format (session id left unset, to be filled in by the caller).
impl From<&[f64]> for Speeds {
    fn from(value: &[f64]) -> Self {
        let mut records = Vec::new();

        value.to_vec().iter().for_each(|speed| {
            records.extend_from_slice(&(*speed).to_be_bytes());
        });

        Speeds {
            session: 0_i64,
            records,
        }
    }
}
