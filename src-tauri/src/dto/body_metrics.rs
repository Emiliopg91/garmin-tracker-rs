use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::{dao::body_metrics::BodyMetrics, utils::date_time_utils::DateTimeUtils};

#[derive(Serialize, Deserialize)]
pub struct BodyMetricListItem {
    pub date: String,
    pub weight: f32,
    pub fat_ratio: f32,
    pub lean_mass: f32,
    pub water_ratio: f32,
}

impl From<&BodyMetrics> for BodyMetricListItem {
    fn from(value: &BodyMetrics) -> Self {
        Self {
            date: DateTimeUtils::format_date(value.date),
            weight: value.weight,
            fat_ratio: value.fat_ratio,
            lean_mass: value.lean_mass,
            water_ratio: value.water_ratio,
        }
    }
}

impl TryFrom<&BodyMetricListItem> for BodyMetrics {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: &BodyMetricListItem) -> Result<Self, Self::Error> {
        let naive = NaiveDateTime::parse_from_str(&value.date, "%H:%M %d/%m/%Y")?;
        let local: DateTime<Local> = Local
            .from_local_datetime(&naive)
            .single()
            .ok_or("Wrong date format")?;

        Ok(Self {
            date: local.timestamp(),
            weight: value.weight,
            fat_ratio: value.fat_ratio,
            lean_mass: value.lean_mass,
            water_ratio: value.water_ratio,
        })
    }
}
