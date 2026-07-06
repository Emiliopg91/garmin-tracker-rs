use chrono::{DateTime, Local, TimeZone};
use rusqlite::Row;

use crate::garmin::database::{
    DATABASE_INST,
    errors::{DatabaseError, Result},
};

pub struct Device {
    pub serial: String,
    pub model: String,
    pub last_sync: Option<DateTime<Local>>,
}

impl Device {
    const FIELD_LIST: &str = "serial, model, last_sync";
    const INSERT_MAKRS: &str = "?, ?, NULL";

    pub fn find_by_id(serial: &str) -> Result<Option<Self>> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;
        let result = conn.query_row(
            &format!("SELECT {} FROM DEVICE WHERE serial=?", Self::FIELD_LIST),
            [&serial],
            Self::map_from_row,
        );

        match result {
            Ok(device) => Ok(Some(device)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DatabaseError::Select(e)),
        }
    }

    pub fn insert(&self) -> Result<()> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;
        conn.execute(
            &format!(
                "INSERT OR IGNORE INTO DEVICE({}) VALUES ({})",
                Self::FIELD_LIST,
                Self::INSERT_MAKRS
            ),
            (&self.serial, &self.model),
        )
        .map_err(DatabaseError::Insert)?;

        Ok(())
    }

    pub fn update_latest_sync(serial: &str, timestamp: DateTime<Local>) -> Result<()> {
        let mut db = DATABASE_INST.lock().unwrap();
        let conn = db.get_connection()?;
        conn.execute(
            "UPDATE DEVICE SET last_sync=? WHERE serial=?",
            (timestamp.timestamp(), &serial),
        )
        .map_err(DatabaseError::Insert)?;

        Ok(())
    }

    fn map_from_row(row: &Row) -> std::result::Result<Self, rusqlite::Error> {
        Ok(Self {
            serial: row.get::<_, String>("serial")?,
            model: row.get::<_, String>("model")?,
            last_sync: row
                .get::<_, Option<i64>>("last_sync")?
                .and_then(|ts| Local.timestamp_opt(ts, 0).single()),
        })
    }
}
