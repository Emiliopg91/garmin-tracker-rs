use std::fmt::Display;
use std::hash::Hash;

use rusqlite::Row;

use crate::garmin::database::{
    DATABASE_INST,
    errors::{DatabaseError, Result},
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
