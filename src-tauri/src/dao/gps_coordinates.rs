use rusqlite_orm_macros::Entity;

#[derive(Clone, Entity)]
#[entity("gps_coordinates")]
#[primary_key(session)]
pub struct GpsCoordinates {
    pub session: i64,
    pub records: Vec<u8>,
}
