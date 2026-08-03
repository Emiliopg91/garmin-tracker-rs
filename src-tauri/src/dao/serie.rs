use std::collections::{HashMap, HashSet};

use chrono::{Datelike, Local, TimeZone, Timelike};
use indexmap::IndexMap;
use rusqlite_orm::{
    dao::{
        Repository,
        helpers::types::{order_by::OrderBy, value::Value, where_clause::Where},
    },
    rusqlite,
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

    pub fn format_date(&self) -> String {
        let datetime = Local.timestamp_opt(self.session, 0).unwrap();
        format!(
            "{}:{} {}/{}/{}",
            datetime.hour(),
            datetime.minute(),
            datetime.day(),
            datetime.month(),
            datetime.year()
        )
    }

    pub fn update_reps_and_weight(
        &self,
        tx: &rusqlite::Transaction,
    ) -> rusqlite_orm::database::errors::Result<usize> {
        SerieRepository::update()
            .set(entity::columns::REPS, self.reps.into())
            .set(entity::columns::WEIGHT, self.weight.into())
            .where_(Where::And(vec![
                Where::Eq(entity::columns::SESSION, self.session.into()),
                Where::Eq(entity::columns::IDX, self.idx.into()),
            ]))
            .execute_in_tx(tx)
    }

    pub fn update_pr(tx: &rusqlite::Transaction, category: &str, id: u16) {
        if let Ok(new_prs) = SerieRepository::select_by_ex_cat_and_ex_id_in_tx(
            tx,
            category,
            id,
            Some(&[
                OrderBy::Desc(entity::columns::WEIGHT),
                OrderBy::Desc(entity::columns::REPS),
                OrderBy::Asc(entity::columns::SESSION),
            ]),
        ) {
            let _ = SerieRepository::update()
                .set(entity::columns::PR, false.into())
                .where_(Where::And(vec![
                    Where::Eq(entity::columns::EX_CAT, category.to_string().into()),
                    Where::Eq(entity::columns::EX_ID, id.into()),
                ]))
                .execute_in_tx(tx);

            for mut pr in new_prs {
                pr.pr = true;
                let _ = pr.update_by_id_in_tx(tx);
            }
        }
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

    pub fn get_pr_for_exercise(
        exercise: &Exercise,
    ) -> rusqlite_orm::database::errors::Result<Serie> {
        Ok(SerieRepository::select()
            .where_(Where::And(vec![
                Where::Eq(entity::columns::EX_CAT, exercise.category.clone().into()),
                Where::Eq(entity::columns::EX_ID, exercise.id.into()),
                Where::Eq(entity::columns::PR, true.into()),
            ]))
            .limit(1)
            .fetch_one()?
            .unwrap())
    }

    pub fn get_prs() -> rusqlite_orm::database::errors::Result<Vec<Serie>> {
        SerieRepository::select()
            .where_(Where::Eq(entity::columns::PR, true.into()))
            .fetch()
    }
}
