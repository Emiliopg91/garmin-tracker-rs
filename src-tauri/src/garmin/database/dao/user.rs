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
    const FIELD_LIST: &str = "date, weight, fat_ratio, lean_mass, water_ratio";
    const INSERT_MARKS: &str = "?, ?, ?, ?, ?";

    pub fn select_all() -> crate::garmin::database::errors::Result<Vec<User>> {
        let mut res = Vec::new();

        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM USER ORDER BY date DESC",
                Self::FIELD_LIST
            ))
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
            &format!(
                "INSERT INTO USER({}) VALUES({})",
                Self::FIELD_LIST,
                Self::INSERT_MARKS
            ),
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
            date: Local.timestamp_opt(row.get::<_, i64>("date")?, 0).unwrap(),
            weight: row.get::<_, f32>("weight")?,
            fat_ratio: row.get::<_, f32>("fat_ratio")?,
            lean_mass: row.get::<_, f32>("lean_mass")?,
            water_ratio: row.get::<_, f32>("water_ratio")?,
        })
    }
}
