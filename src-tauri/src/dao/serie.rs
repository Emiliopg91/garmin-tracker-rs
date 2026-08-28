use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

use crate::dao::exercise::{self};

use super::exercise::Exercise;

#[derive(Entity, Clone, Serialize, Deserialize)]
#[primary_key(session, idx)]
#[index("session", (session))]
#[index("personal_records", (pr))]
#[index("exercise", (ex_cat, ex_id))]
#[unique("exercise_personal_record", (ex_cat, ex_id), (pr=true))]
pub struct Serie {
    #[serde(skip)]
    pub session: i64,
    pub idx: u8,
    #[column("exercise_category")]
    pub ex_cat: String,
    #[column("exercise_id")]
    pub ex_id: u16,
    pub reps: u16,
    pub weight: f64,
    pub pr: bool,

    #[serde(skip)]
    #[relationship((ex_cat, exercise::entity::columns::CATEGORY),(ex_id, exercise::entity::columns::ID))]
    pub exercise: Option<Exercise>,
}
