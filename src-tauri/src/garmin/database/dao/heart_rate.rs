use garmin_tracker_rs_macros::Entity;

#[derive(Entity, Debug, Clone)]
#[entity(table = "heart_rate")]
pub struct HeartRate {
    #[id]
    pub session: i64,
    #[id]
    pub idx: u32,
    pub hr: u8,
}
