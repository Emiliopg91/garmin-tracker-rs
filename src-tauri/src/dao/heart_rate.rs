use rusqlite_orm_macros::Entity;

#[derive(Clone, Entity)]
#[entity("heart_rate")]
#[primary_key(session)]
pub struct HeartRate {
    pub session: i64,
    pub records: Vec<u8>,
}
