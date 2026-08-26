use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

#[derive(Entity, Serialize, Deserialize)]
#[entity("body_metrics")]
#[primary_key(date)]
pub struct BodyMetrics {
    pub date: i64,
    pub weight: f32,
    pub fat_ratio: f32,
    pub lean_mass: f32,
    pub water_ratio: f32,
}
