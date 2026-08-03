use rusqlite_orm_macros::Entity;

#[derive(Entity, Clone, Default)]
#[entity(table = "heart_rate")]
#[indexes((session))]
pub struct HeartRate {
    #[primary_key]
    pub session: i64,
    #[primary_key]
    pub idx: u32,
    pub hr: u8,
}
