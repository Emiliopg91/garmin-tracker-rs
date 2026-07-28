use garmin_tracker_rs_macros::Entity;

#[derive(Default, Clone, Entity)]
#[indexes((serial))]
pub struct Device {
    #[id]
    pub serial: String,
    pub model: String,
    pub last_sync: Option<i64>,
}
