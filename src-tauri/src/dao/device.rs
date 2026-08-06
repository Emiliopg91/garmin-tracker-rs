use rusqlite_orm_macros::Entity;

#[derive(Clone, Entity)]
#[primary_key(serial)]
pub struct Device {
    pub serial: String,
    pub model: String,
    pub last_sync: Option<i64>,
}
