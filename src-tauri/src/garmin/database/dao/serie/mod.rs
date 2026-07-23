use std::fmt::Display;

use chrono::{Datelike, Local, TimeZone, Timelike};
use garmin_tracker_rs_macros::Entity;
use indexmap::IndexMap;

use crate::garmin::database::dao::{
    Entity,
    helpers::types::{order_by::OrderBy, where_clause::Where},
};

use super::exercise::Exercise;

#[derive(Debug, Default, Entity, Clone)]
pub struct Serie {
    #[id]
    pub session: i64,
    #[id]
    pub idx: u8,
    pub exercise_category: String,
    pub exercise_id: u16,
    pub reps: u16,
    pub weight: f64,
    pub pr: bool,
}
impl Display for Serie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}Kg", self.reps, self.weight)
    }
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
    ) -> crate::garmin::database::errors::Result<()> {
        Serie::update()
            .set(SERIE_COLUMN_REPS, self.reps.into())
            .set(SERIE_COLUMN_WEIGHT, self.weight.into())
            .where_(Where::And(vec![
                Where::Eq(SERIE_COLUMN_SESSION, self.session.into()),
                Where::Eq(SERIE_COLUMN_IDX, self.idx.into()),
            ]))
            .execute_in_transaction(tx)
    }

    pub fn update_pr(tx: &rusqlite::Transaction, category: &str, id: u16) {
        let result = Serie::select()
            .where_(Where::And(vec![
                Where::Eq(SERIE_COLUMN_EXERCISE_CATEGORY, category.into()),
                Where::Eq(SERIE_COLUMN_EXERCISE_ID, id.into()),
            ]))
            .order_by(OrderBy::Desc(SERIE_COLUMN_WEIGHT))
            .order_by(OrderBy::Desc(SERIE_COLUMN_REPS))
            .fetch_one_in_transaction(tx)
            .map(|rs| rs.unwrap());

        if let Ok(serie) = result {
            let _ = Serie::update()
                .set(SERIE_COLUMN_PR, false.into())
                .where_(Where::And(vec![
                    Where::Eq(SERIE_COLUMN_EXERCISE_CATEGORY, category.to_string().into()),
                    Where::Eq(SERIE_COLUMN_EXERCISE_ID, id.into()),
                ]))
                .execute_in_transaction(tx);

            let _ = Serie::update()
                .set(SERIE_COLUMN_PR, true.into())
                .where_(Where::And(vec![
                    Where::Eq(SERIE_COLUMN_SESSION, serie.session.into()),
                    Where::Eq(SERIE_COLUMN_IDX, serie.idx.into()),
                ]))
                .execute_in_transaction(tx);
        }
    }

    pub fn load_for_session(
        session: i64,
    ) -> crate::garmin::database::errors::Result<IndexMap<Exercise, Vec<Serie>>> {
        let mut res = IndexMap::new();
        let tuple_rows = Serie::select()
            .where_(Where::Eq(SERIE_COLUMN_SESSION, session.into()))
            .order_by(OrderBy::Asc(SERIE_COLUMN_IDX))
            .fetch()?;

        for r in tuple_rows {
            let ex = Exercise {
                category: r.exercise_category.clone(),
                id: r.exercise_id,
                name: "".to_string(),
            };
            if !res.contains_key(&ex) {
                if let Some(ex) = Exercise::select_by_id(ex.id, ex.category.clone())? {
                    res.insert(ex, Vec::new());
                } else {
                    continue;
                }
            }
            res.get_mut(&ex).unwrap().push(r);
        }

        Ok(res)
    }

    pub fn get_pr_for_exercise(
        exercise: &Exercise,
    ) -> crate::garmin::database::errors::Result<Serie> {
        Ok(Serie::select()
            .where_(Where::And(vec![
                Where::Eq(
                    SERIE_COLUMN_EXERCISE_CATEGORY,
                    exercise.category.clone().into(),
                ),
                Where::Eq(SERIE_COLUMN_EXERCISE_ID, exercise.id.into()),
            ]))
            .limit(1)
            .fetch()?
            .into_iter()
            .next()
            .unwrap())
    }
}
