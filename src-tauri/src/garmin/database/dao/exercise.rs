use std::fmt::Display;
use std::hash::Hash;

use fitparser::{profile, FitDataRecord, Value};
use rusqlite::Row;

use crate::garmin::database::{
    dao::errors::{self, ParseFitFileError},
    errors::{DatabaseError, Result},
    DATABASE_INST,
};

#[derive(Clone, Debug)]
pub struct Exercise {
    pub id: u16,
    pub category: String,
    pub name: String,
}
impl Display for Exercise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl PartialEq for Exercise {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.category == other.category
    }
}
impl Eq for Exercise {}
impl Hash for Exercise {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.category.hash(state);
    }
}

impl Exercise {
    pub(crate) fn get_exercises(entries: &[FitDataRecord]) -> errors::Result<Vec<Exercise>> {
        let mut exercises = Vec::new();

        entries
            .iter()
            .filter(|r| r.kind() == profile::MesgNum::ExerciseTitle)
            .try_for_each(|reg| -> errors::Result<()> {
                let ex_id = reg.fields().iter().find_map(|r| {
                    if r.name() == "exercise_name" {
                        if let Value::UInt16(val) = r.value() {
                            Some(val)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                let ex_id = ex_id.unwrap_or(&1);

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

                if ex_cat.is_none() {
                    return Err(ParseFitFileError::InvalidFileFormat(
                        "Missing exercise category field".to_string(),
                    ));
                }
                let ex_cat = ex_cat.unwrap();

                let name = reg.fields().iter().find_map(|r| {
                    if r.name() == "wkt_step_name" {
                        if let Value::String(val) = r.value() {
                            Some(val)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                if name.is_none() {
                    return Err(ParseFitFileError::InvalidFileFormat(
                        "Missing exercise name field".to_string(),
                    ));
                }
                let name = name.unwrap();

                exercises.push(Exercise {
                    id: *ex_id,
                    category: ex_cat.clone(),
                    name: name.clone(),
                });

                Ok(())
            })?;

        Ok(exercises)
    }

    pub fn insert(
        &self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<()> {
        tx.execute(
            "INSERT OR IGNORE INTO EXERCISE VALUES(?,?,?)",
            (&self.category, self.id, &self.name),
        )
        .map_err(DatabaseError::Insert)?;

        Ok(())
    }

    pub fn load_by_cat_and_id(category: &str, id: u16) -> Result<Option<Exercise>> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT * FROM EXERCISE WHERE category=? AND id=?")
            .unwrap();

        let rows = stmt
            .query_map((&category, &id), Self::map_from_row)
            .map_err(DatabaseError::Select)?;

        let mut inst = None;
        rows.for_each(|r| {
            if let Ok(r) = r {
                inst = Some(r);
            }
        });

        Ok(inst)
    }

    fn map_from_row(row: &Row) -> std::result::Result<Self, rusqlite::Error> {
        Ok(Self {
            category: row.get::<_, String>(0)?,
            id: row.get::<_, u16>(1)?,
            name: row.get::<_, String>(2)?,
        })
    }

    pub fn load_from_db() -> Result<Vec<Exercise>> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;

        let mut stmt = conn
            .prepare("SELECT * FROM EXERCISE ORDER BY name ASC")
            .unwrap();
        let rows = stmt.query_map((), Self::map_from_row).unwrap();

        let mut res = Vec::new();
        rows.for_each(|r| {
            if let Ok(r) = r {
                res.push(r);
            }
        });
        Ok(res)
    }
}
