use rusqlite_orm_macros::Entity;

#[derive(Clone, Entity)]
#[entity("gps_coordinates")]
#[primary_key(session)]
pub struct GpsCoordinates {
    pub session: i64,
    pub records: Vec<u8>,
}

impl GpsCoordinates {
    pub fn normalize(&self) -> Vec<(i32, i32)> {
        self.records
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
