use garmin_tracker_rs_macros::Entity;

#[derive(Default, Entity)]
pub struct Device {
    #[id]
    pub serial: String,
    pub model: String,
    pub last_sync: Option<i64>,
}
