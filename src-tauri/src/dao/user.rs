use chrono::{Datelike, Local, TimeZone};
use rusqlite_orm_macros::Entity;

#[derive(Default, Entity)]
#[indexes((date))]
pub struct User {
    #[primary_key]
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
