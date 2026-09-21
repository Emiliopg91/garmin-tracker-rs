use rusqlite_orm::Entity;
use serde::{Deserialize, Serialize};

use crate::dao::exercise::{self};

use super::exercise::Exercise;

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(session, idx)]
#[index("session", (session))]
#[index("personal_records", (pr))]
#[index("exercise", (ex_cat, ex_id))]
#[unique("exercise_personal_record", (ex_cat, ex_id), (pr=true))]
pub struct Set {
    pub session: i64,
    pub idx: u8,
    #[column("exercise_category")]
    pub ex_cat: u16,
    #[column("exercise_id")]
    pub ex_id: u16,
    pub reps: u16,
    pub weight: f64,
    pub pr: bool,
    pub e1rm: i32,

    #[relationship((ex_cat, exercise::entity::columns::CATEGORY),(ex_id, exercise::entity::columns::ID))]
    #[allow(dead_code)]
    pub exercise: Option<Exercise>,
}

impl Set {
    pub fn estimate_1rm(weight: f64, reps: u16) -> i32 {
        if reps <= 1 {
            return weight as i32;
        }

        let r = reps as f64;

        if reps <= 10 && 37.0 - r > 0.0 {
            return ((weight * 36.0) / (37.0 - r)) as i32;
        }

        if reps <= 20 {
            let denom = 101.3 - 2.67123 * r;
            if denom > 0.0 {
                return ((100.0 * weight) / denom) as i32;
            } else {
                return (weight * (1.0 + r / 30.0)) as i32;
            }
        }

        (weight * r.powf(0.1)) as i32
    }
}
