use std::fmt::Display;

use chrono::{DateTime, Local, TimeZone};
use fitparser::{FitDataRecord, Value, profile};
use indexmap::IndexMap;
use rusqlite::{Row, types::Type};

use crate::garmin::database::{
    DATABASE_INST,
    dao::errors,
    errors::{DatabaseError, Result},
};

use super::{exercise::Exercise, session::Session};

#[derive(Debug, Default)]
pub struct Serie {
    pub session: DateTime<Local>,
    pub idx: u8,
    pub exercise_category: String,
    pub exercise_id: u16,
    pub reps: u16,
    pub weight: f64,
}
impl Display for Serie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}Kg", self.reps, self.weight)
    }
}

impl Serie {
    fn get_steps(entries: &[FitDataRecord], exercises: &[Exercise]) -> Vec<Option<Exercise>> {
        let mut steps = Vec::new();

        let mut idx = 0;
        entries
            .iter()
            .filter(|r| r.kind() == profile::MesgNum::WorkoutStep)
            .for_each(|reg| {
                let mut step = None;

                let ex_cat = reg.fields().iter().find_map(|r| {
                    if r.name() == "exercise_category" {
                        if let Value::String(val) = r.value() {
                            Some(val)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                if let Some(ex_cat) = ex_cat {
                    let ex_id = reg
                        .fields()
                        .iter()
                        .find_map(|r| {
                            if r.name() == "exercise_name" {
                                if let Value::UInt16(val) = r.value() {
                                    Some(val)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .unwrap_or(&1);

                    let exercise = exercises
                        .iter()
                        .find(|e| e.id == *ex_id && e.category == *ex_cat)
                        .expect("Undefined exercise from step");

                    step = Some((*exercise).clone());
                }

                steps.push(step);
                idx += 1;
            });

        steps
    }

    pub(crate) fn get_sets(
        entries: &[FitDataRecord],
        session: &Session,
    ) -> errors::Result<IndexMap<Exercise, Vec<Serie>>> {
        let exercises = Exercise::get_exercises(entries)?;

        let steps = Self::get_steps(entries, &exercises);

        let mut sets = IndexMap::new();

        let mut idx = 0_u8;
        entries
            .iter()
            .filter(|r| r.kind() == profile::MesgNum::Set)
            .for_each(|reg| {
                if reg.kind() == profile::MesgNum::Set {
                    let reps = reg.fields().iter().find_map(|r| {
                        if r.name() == "repetitions" {
                            if let Value::UInt16(val) = r.value() {
                                Some(val)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    });

                    if let Some(reps) = reps {
                        let weight = reg.fields().iter().find_map(|r| {
                            if r.name() == "weight" {
                                if let Value::Float64(val) = r.value() {
                                    Some(val)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        });

                        if let Some(weight) = weight {
                            let ex_idx = reg.fields().iter().find_map(|r| {
                                if r.name() == "wkt_step_index" {
                                    if let Value::SInt64(val) = r.value() {
                                        Some(val)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            });

                            if let Some(ex_idx) = ex_idx
                                && let Some(exercise) = steps.get(*ex_idx as usize)
                                && let Some(exercise) = exercise
                            {
                                let entry: &mut Vec<Serie> =
                                    sets.entry(exercise.clone()).or_default();

                                entry.push(Serie {
                                    session: session.timestamp,
                                    idx: idx as u8,
                                    exercise_category: exercise.category.clone(),
                                    exercise_id: exercise.id,
                                    reps: *reps,
                                    weight: *weight,
                                });
                                idx += 1;
                            }
                        }
                    }
                }
            });

        Ok(sets)
    }

    pub fn insert(
        &self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<()> {
        println!("{} {}", self.session.timestamp(), self.idx);
        tx.execute(
            "INSERT INTO SERIE VALUES(?,?,?,?,?,?)",
            (
                self.session.timestamp(),
                self.idx,
                self.exercise_category.clone(),
                self.exercise_id,
                self.reps,
                self.weight,
            ),
        )
        .map_err(DatabaseError::Insert)?;

        Ok(())
    }

    pub fn load_for_session(session: DateTime<Local>) -> Result<IndexMap<Exercise, Vec<Serie>>> {
        let mut res = IndexMap::new();
        let tuple_rows: Vec<_>;
        {
            let mut db = DATABASE_INST.lock().unwrap();
            let conn = db.get_connection()?;

            let mut stmt = conn
                .prepare("SELECT * FROM SERIE WHERE session=? ORDER BY idx")
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

    pub fn load_for_exercise(category: &str, id: i16) -> Result<Vec<Serie>> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;

        let mut stmt = conn
                .prepare("SELECT * FROM SERIE WHERE exercise_category=? and exercise_id=? ORDER BY session DESC")
                .map_err(DatabaseError::Select)?;
        let rows = stmt
            .query_map((category, id), Self::map_from_row)
            .map_err(DatabaseError::Select)?;
        Ok(rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(DatabaseError::Select)?)
    }

    fn map_from_row(row: &Row) -> std::result::Result<Self, rusqlite::Error> {
        Ok(Self {
            session: Local.timestamp_opt(row.get::<_, i64>(0)?, 0).unwrap(),
            idx: row.get::<_, u8>(1)?,
            exercise_category: row.get::<_, String>(2)?,
            exercise_id: row.get::<_, u16>(3)?,
            reps: row.get::<_, u16>(4)?,
            weight: row.get::<_, f64>(5)?,
        })
    }

    pub fn get_pr_for_exercise(exercise: &Exercise) -> Result<Serie> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;

        let mut stmt = conn.prepare("SELECT * FROM SERIE WHERE exercise_category=? AND exercise_id=? ORDER BY weight DESC, reps DESC LIMIT 1").unwrap();
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
