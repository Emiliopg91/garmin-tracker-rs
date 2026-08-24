use serde::{Deserialize, Serialize};

use crate::dao::body_metrics::BodyMetrics;

#[derive(Serialize, Deserialize)]
pub struct BodyMetricListItem {
    pub date: i32,
    pub weight: f32,
    pub fat_ratio: f32,
    pub lean_mass: f32,
    pub water_ratio: f32,
}

impl From<&BodyMetrics> for BodyMetricListItem {
    fn from(value: &BodyMetrics) -> Self {
        Self {
            date: value.date as i32,
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
        Ok(Self {
            date: value.date as i64,
            weight: value.weight,
            fat_ratio: value.fat_ratio,
            lean_mass: value.lean_mass,
            water_ratio: value.water_ratio,
        })
    }
}
