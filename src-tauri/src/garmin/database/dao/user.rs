use chrono::{Datelike, Local, TimeZone};
use garmin_tracker_rs_macros::Entity;

#[derive(Default, Entity)]
pub struct User {
    #[id]
    pub date: i64,
    pub weight: f32,
    pub fat_ratio: f32,
    pub lean_mass: f32,
    pub water_ratio: f32,
}

impl User {
    pub fn format_date(&self) -> String {
        let datetime = Local.timestamp_opt(self.date, 0).unwrap();
        format!(
            "{:02}/{:02}/{:04}",
            datetime.day(),
            datetime.month(),
            datetime.year()
        )
    }
}
