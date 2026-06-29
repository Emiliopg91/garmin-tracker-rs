use chrono::{DateTime, Datelike, Local, TimeZone};
use rusqlite::Row;

use crate::garmin::database::{DATABASE_INST, errors::DatabaseError};

pub struct User {
    pub date: DateTime<Local>,
    pub weight: f32,
    pub fat_ratio: f32,
    pub lean_mass: f32,
    pub water_ratio: f32,
}

impl User {
    pub fn select_all() -> crate::garmin::database::errors::Result<Vec<User>> {
        let mut res = Vec::new();

        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT * FROM USER ORDER BY date DESC")
            .map_err(DatabaseError::Select)?;

        let rows = stmt
            .query_map((), Self::map_from_row)
            .map_err(DatabaseError::Select)?;

        rows.for_each(|r| {
            if let Ok(r) = r {
                res.push(r);
            }
        });

        Ok(res)
    }

    pub fn insert(&self) -> crate::garmin::database::errors::Result<()> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;
        conn.execute(
            "INSERT INTO USER VALUES(?,?,?,?,?)",
            (
                self.date.timestamp(),
                self.weight,
                self.fat_ratio,
                self.lean_mass,
                self.water_ratio,
            ),
        )
        .map_err(DatabaseError::Insert)
        .map(|_| ())
    }

    pub fn format_date(&self) -> String {
        format!(
            "{:02}/{:02}/{:04}",
            self.date.day(),
            self.date.month(),
            self.date.year()
        )
    }

    fn map_from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            date: Local.timestamp_opt(row.get::<_, i64>(0)?, 0).unwrap(),
            weight: row.get::<_, f32>(1)?,
            fat_ratio: row.get::<_, f32>(2)?,
            lean_mass: row.get::<_, f32>(3)?,
            water_ratio: row.get::<_, f32>(4)?,
        })
    }
}
