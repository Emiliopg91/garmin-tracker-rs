use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use rusqlite_orm::dao::{
    Repository,
    helpers::types::{order_by::OrderBy, value::Value, where_clause::Where},
};
use rusqlite_orm_macros::Entity;

use crate::dao::exercise::{self, ExerciseRepository};

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
}

impl Serie {
    pub fn get_1rm_estimation(&self) -> f64 {
        self.weight * (self.reps as f64).powf(0.1)
    }

    pub fn load_for_session(
        session: i64,
    ) -> rusqlite_orm::database::errors::Result<IndexMap<Exercise, Vec<Serie>>> {
        let tuple_rows = SerieRepository::select_by_session(
            session,
            Some(&[OrderBy::Asc(entity::columns::IDX)]),
        )?;

        let condition_set: HashSet<(_, _)> = tuple_rows
            .iter()
            .map(|r| (r.ex_cat.clone(), r.ex_id))
            .collect();

        let in_conditions = condition_set
            .into_iter()
            .map(|(cat, id)| vec![cat.into(), id.into()])
            .collect::<Vec<Vec<Value>>>();

        let exercises = ExerciseRepository::select()
            .where_(Where::InMultiple(
                vec![
                    exercise::entity::columns::CATEGORY,
                    exercise::entity::columns::ID,
                ],
                in_conditions,
            ))
            .fetch()?;

        let exercise_by_key: HashMap<(_, _), &Exercise> = exercises
            .iter()
            .map(|e| ((e.category.clone(), e.id), e))
            .collect();

        let mut res: IndexMap<Exercise, Vec<Serie>> = IndexMap::with_capacity(exercises.len());

        for r in tuple_rows {
            if let Some(&ex) = exercise_by_key.get(&(r.ex_cat.clone(), r.ex_id)) {
                res.entry(ex.clone()).or_default().push(r);
            }
        }

        Ok(res)
    }
}
