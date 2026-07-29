use rusqlite_orm_macros::Entity;

#[derive(Default, Clone, Entity)]
pub struct Device {
    #[primary_key]
    pub serial: String,
    pub model: String,
    pub last_sync: Option<i64>,
}
