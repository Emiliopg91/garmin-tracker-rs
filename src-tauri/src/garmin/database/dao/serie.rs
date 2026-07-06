use std::fmt::Display;

use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use indexmap::IndexMap;
use rusqlite::Row;

use crate::garmin::database::{
    DATABASE_INST,
    errors::{DatabaseError, Result},
};

use super::exercise::Exercise;

#[derive(Debug, Default)]
pub struct Serie {
    pub session: DateTime<Local>,
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
    const FIELD_LIST: &str = "session, idx, exercise_category, exercise_id, reps, weight, pr";

    pub fn format_date(&self) -> String {
        format!(
            "{}:{} {}/{}/{}",
            self.session.hour(),
            self.session.minute(),
            self.session.day(),
            self.session.month(),
            self.session.year()
        )
    }

    pub fn insert(
        &mut self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<()> {
        tx.execute(
            &format!(
                "INSERT INTO SERIE({}) VALUES(?,?,?,?,?,?,?)",
                Self::FIELD_LIST
            ),
            (
                self.session.timestamp(),
                self.idx,
                self.exercise_category.clone(),
                self.exercise_id,
                self.reps,
                self.weight,
                self.pr,
            ),
        )
        .map_err(DatabaseError::Insert)?;

        Ok(())
    }

    pub fn update_pr(tx: &rusqlite::Transaction, category: &str, id: u16) {
        let result = tx.query_one(
            &format!("SELECT {} FROM SERIE WHERE exercise_category=? AND exercise_id=? ORDER BY weight DESC, reps DESC LIMIT 1", Self::FIELD_LIST),
            (&category, id),
            Self::map_from_row,
        );

        if let Ok(serie) = result {
            let _ = tx.execute(
                "UPDATE SERIE SET pr=0 WHERE exercise_category=? AND exercise_id=?",
                (&category, id),
            );
            let _ = tx.execute(
                "UPDATE SERIE SET pr=1 WHERE session=? AND idx=?",
                (serie.session.timestamp(), serie.idx),
            );
        }
    }

    pub fn update_serie(&self, tx: &rusqlite::Transaction) {
        let _ = tx.execute(
            "UPDATE SERIE SET reps=?, weight=? WHERE session=? AND idx=?",
            (self.reps, self.weight, &self.session.timestamp(), self.idx),
        );
    }

    pub fn load_for_session(session: DateTime<Local>) -> Result<IndexMap<Exercise, Vec<Serie>>> {
        let mut res = IndexMap::new();
        let tuple_rows: Vec<_>;
        {
            let mut db = DATABASE_INST.lock().unwrap();
            let conn = db.get_connection()?;

            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {} FROM SERIE WHERE session=? ORDER BY idx",
                    Self::FIELD_LIST
                ))
                .map_err(DatabaseError::Select)?;
            let rows = stmt
                .query_map([session.timestamp()], Self::map_from_row)
                .map_err(DatabaseError::Select)?;
            tuple_rows = rows
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(DatabaseError::Select)?;
        }

        for r in tuple_rows {
            let ex = Exercise {
                category: r.exercise_category.clone(),
                id: r.exercise_id,
                name: "".to_string(),
            };
            if !res.contains_key(&ex) {
                if let Some(ex) = Exercise::load_by_cat_and_id(&ex.category, ex.id)? {
                    res.insert(ex, Vec::new());
                } else {
                    continue;
                }
            }
            res.get_mut(&ex).unwrap().push(r);
        }

        Ok(res)
    }

    pub fn load_for_session_and_idx(session: i64, idx: u8) -> Result<Option<Serie>> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;

        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM SERIE WHERE session=? AND idx=?",
                Self::FIELD_LIST
            ))
            .map_err(DatabaseError::Select)?;
        let rows = stmt
            .query_map((session, idx), Self::map_from_row)
            .map_err(DatabaseError::Select)?;

        if let Some(r) = rows.flatten().next() {
            return Ok(Some(r));
        }
        Ok(None)
    }

    pub fn load_for_exercise(category: &str, id: i16) -> Result<Vec<Serie>> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;

        let mut stmt = conn
                .prepare(&format!("SELECT {} FROM SERIE WHERE exercise_category=? and exercise_id=? ORDER BY session DESC", Self::FIELD_LIST))
                .map_err(DatabaseError::Select)?;
        let rows = stmt
            .query_map((category, id), Self::map_from_row)
            .map_err(DatabaseError::Select)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DatabaseError::Select)
    }

    fn map_from_row(row: &Row) -> std::result::Result<Self, rusqlite::Error> {
        Ok(Self {
            session: Local
                .timestamp_opt(row.get::<_, i64>("session")?, 0)
                .unwrap(),
            idx: row.get::<_, u8>("idx")?,
            exercise_category: row.get::<_, String>("exercise_category")?,
            exercise_id: row.get::<_, u16>("exercise_id")?,
            reps: row.get::<_, u16>("reps")?,
            weight: row.get::<_, f64>("weight")?,
            pr: row.get::<_, bool>("pr")?,
        })
    }

    pub fn get_pr_for_exercise(exercise: &Exercise) -> Result<Serie> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;

        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM SERIE WHERE exercise_category=? AND exercise_id=? AND pr=TRUE",
                Self::FIELD_LIST
            ))
            .unwrap();
        let rows = stmt
            .query_map((&exercise.category, &exercise.id), Self::map_from_row)
            .unwrap();

        let mut res = Serie::default();
        rows.for_each(|r| {
            if let Ok(r) = r {
                res = r;
            }
        });
        Ok(res)
    }

    pub fn get_1rm_estimation(&self) -> f64 {
        self.weight * (self.reps as f64).powf(0.1)
    }
}
