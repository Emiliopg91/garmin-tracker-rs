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
    /// Upper bound of reps used for the estimation: beyond this the formulas are unreliable,
    /// so higher-rep sets are estimated as if they were done at this count.
    const MAX_ESTIMATION_REPS: u16 = 20;

    /// Estimates the one-rep max: Brzycki up to 10 reps, Lander from 11 to 20.
    pub fn estimate_1rm(weight: f64, reps: u16) -> i32 {
        if reps <= 1 {
            return weight as i32;
        }

        let r = reps.min(Self::MAX_ESTIMATION_REPS) as f64;

        if r <= 10.0 {
            ((weight * 36.0) / (37.0 - r)) as i32
        } else {
            ((100.0 * weight) / (101.3 - 2.67123 * r)) as i32
        }
    }
}
