use rusqlite_orm_macros::Entity;

#[derive(Entity, Clone)]
#[entity(table = "heart_rate")]
#[indexes((session))]
pub struct HeartRate {
    #[id]
    pub session: i64,
    #[id]
    pub idx: u32,
    pub hr: u8,
}
