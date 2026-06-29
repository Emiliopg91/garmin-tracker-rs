use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::garmin::database::dao::user::User;

#[derive(Serialize, Deserialize)]
pub struct UserListItem {
    pub date: String,
    pub weight: f32,
    pub fat_ratio: f32,
    pub lean_mass: f32,
    pub water_ratio: f32,
}

impl From<&User> for UserListItem {
    fn from(value: &User) -> Self {
        Self {
            date: value.format_date(),
            weight: value.weight,
            fat_ratio: value.fat_ratio,
            lean_mass: value.lean_mass,
            water_ratio: value.water_ratio,
        }
    }
}

impl TryFrom<&UserListItem> for User {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &UserListItem) -> Result<Self, Self::Error> {
        let naive = NaiveDateTime::parse_from_str(&value.date, "%H:%M %d/%m/%Y")?;
        let local: DateTime<Local> = Local
            .from_local_datetime(&naive)
            .single()
            .ok_or("Fecha/hora ambigua o inexistente")?;

        Ok(Self {
            date: local,
            weight: value.weight,
            fat_ratio: value.fat_ratio,
            lean_mass: value.lean_mass,
            water_ratio: value.water_ratio,
        })
    }
}
