use std::fmt::Display;

use chrono::{DateTime, Local};
use fitparser::{FitDataRecord, Value, profile};
use indexmap::IndexMap;
use rusqlite::Row;

use crate::garmin::{
    database::{
        DATABASE_INST,
        errors::{DatabaseError, Result},
    },
    models::errors,
};

use super::{exercise::Exercise, session::Session};

#[derive(Debug, Default)]
pub struct Serie {
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
    ) -> errors::Result<IndexMap<Exercise, Vec<Serie>>> {
        let exercises = Exercise::get_exercises(entries)?;

        let steps = Self::get_steps(entries, &exercises);

        let mut sets = IndexMap::new();

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
                                    reps: *reps,
                                    weight: *weight,
                                });
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
        session: &Session,
        exercise: &Exercise,
        pos: u8,
    ) -> crate::garmin::database::errors::Result<()> {
        tx.execute(
            "INSERT INTO SERIE VALUES(?,?,?,?,?,?)",
            (
                session.timestamp.to_rfc3339(),
                pos,
                &exercise.category,
                &exercise.id,
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
                .query_map([session.to_rfc3339()], |row| {
                    Ok((
                        row.get::<_, String>(2)?,
                        row.get::<_, u16>(3)?,
                        row.get::<_, u16>(4)?,
                        row.get::<_, f64>(5)?,
                    ))
                })
                .map_err(DatabaseError::Select)?;
            tuple_rows = rows
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(DatabaseError::Select)?;
        }

        for r in tuple_rows {
            let ex = Exercise {
                category: r.0,
                id: r.1,
                name: "".to_string(),
            };
            if !res.contains_key(&ex) {
                if let Some(ex) = Exercise::load_by_cat_and_id(&ex.category, ex.id)? {
                    res.insert(ex, Vec::new());
                } else {
                    continue;
                }
            }
            let serie = Serie {
                reps: r.2,
                weight: r.3,
            };
            res.get_mut(&ex).unwrap().push(serie);
        }

        Ok(res)
    }

    fn map_from_row(row: &Row) -> std::result::Result<Self, rusqlite::Error> {
        Ok(Self {
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
