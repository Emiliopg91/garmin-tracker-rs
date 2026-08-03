use rusqlite_orm_macros::Entity;

use crate::dao::exercise::{self};

use super::exercise::Exercise;

#[derive(Default, Entity, Clone)]
#[indexes((session), (ex_cat, ex_id))]
pub struct Serie {
    #[primary_key]
    pub session: i64,
    #[primary_key]
    pub idx: u8,
    #[column(name = "exercise_category")]
    pub ex_cat: String,
    #[column(name = "exercise_id")]
    pub ex_id: u16,
    pub reps: u16,
    pub weight: f64,
    pub pr: bool,

    #[relationship((ex_cat, exercise::entity::columns::CATEGORY),(ex_id, exercise::entity::columns::ID))]
    pub exercise: Option<Exercise>,
}

impl Serie {
    pub fn get_1rm_estimation(&self) -> f64 {
        self.weight * (self.reps as f64).powf(0.1)
    }
}
