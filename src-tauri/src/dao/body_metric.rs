use rusqlite_orm::Entity;
use serde::{Deserialize, Serialize};

#[derive(Entity, Serialize, Deserialize)]
#[entity("body_metric")]
#[primary_key(date)]
pub struct BodyMetric {
    pub date: u32,
    pub weight: f32,
    pub fat_ratio: f32,
    pub lean_mass: f32,
    pub water_ratio: f32,
}
