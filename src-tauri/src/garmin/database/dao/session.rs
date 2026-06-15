use std::{
    fmt::Display,
    fs::{self, File},
    path::Path,
};

use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use fitparser::{FitDataRecord, Value, profile};
use indexmap::IndexMap;
use rusqlite::{Row, types::Type};

use crate::garmin::database::{
    DATABASE_INST,
    dao::errors::{self, ParseFitFileError},
    errors::{DatabaseError, Result},
};

use super::{exercise::Exercise, serie::Serie};

#[derive(Default, Debug)]
pub struct Session {
    pub workout: String,

    pub timestamp: DateTime<Local>,

    pub total_elapsed_time: f64,
    pub active_time: f64,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub avg_heart_rate: u8,
    pub max_heart_rate: u8,

    pub series: IndexMap<Exercise, Vec<Serie>>,
}
impl Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.workout, self.format_date())
    }
}
impl Session {
    pub fn format_date(&self) -> String {
        format!(
            "{}:{} {}/{}/{}",
            self.timestamp.hour(),
            self.timestamp.minute(),
            self.timestamp.day(),
            self.timestamp.month(),
            self.timestamp.year()
        )
    }

    pub fn format_total_time(&self) -> String {
        Self::format_duration(self.total_elapsed_time as u64)
    }

    pub fn format_active_time(&self) -> String {
        Self::format_duration(self.active_time as u64)
    }

    fn format_duration(seconds: u64) -> String {
        let h = seconds / 3600;
        let m = (seconds % 3600) / 60;
        let s = seconds % 60;

        if h > 0 {
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else if m > 0 {
            format!("{:02}:{:02}", m, s)
        } else {
            format!("{s}")
        }
    }

    pub fn find_by_id(timestamp: i64) -> Result<Option<Session>> {
        let opt_sess = {
            let mut db = DATABASE_INST.lock().unwrap();
            let conn = db.get_connection()?;

            let result = conn.query_row(
                "SELECT * FROM SESSION WHERE date=?",
                [timestamp],
                Self::map_from_row,
            );

            match result {
                Ok(session) => Some(session),
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(e) => return Err(DatabaseError::Select(e)),
            }
        };

        Ok(match opt_sess {
            Some(mut session) => {
                session.series = Serie::load_for_session(session.timestamp)?;
                Some(session)
            }
            None => None,
        })
    }

    pub fn load_from_db() -> Result<Vec<Session>> {
        let mut res = Vec::new();

        {
            let mut db = DATABASE_INST.lock().unwrap();
            let conn = db.get_connection()?;

            let mut stmt = conn
                .prepare("SELECT * FROM SESSION ORDER BY date DESC")
                .map_err(DatabaseError::Select)?;

            let rows = stmt
                .query_map([], Self::map_from_row)
                .map_err(DatabaseError::Select)?;

            rows.for_each(|r| {
                if let Ok(r) = r {
                    res.push(r);
                }
            });
        }

        for r in &mut res {
            r.series = Serie::load_for_session(r.timestamp)?;
        }

        Ok(res)
    }

    fn map_from_row(row: &Row) -> std::result::Result<Self, rusqlite::Error> {
        Ok(Session {
            timestamp: Local.timestamp_opt(row.get::<_, i64>(0)?, 0).unwrap(),
            workout: row.get::<_, String>(1)?,
            total_elapsed_time: row.get::<_, f64>(2)?,
            active_time: row.get::<_, f64>(3)?,
            total_calories: row.get::<_, u16>(4)?,
            metabolic_calories: row.get::<_, u16>(5)?,
            avg_heart_rate: row.get::<_, u8>(6)?,
            max_heart_rate: row.get::<_, u8>(7)?,
            series: IndexMap::new(),
        })
    }

    pub fn insert(
        &self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<()> {
        tx.execute(
            "INSERT INTO SESSION VALUES(?,?,?,?,?,?,?,?)",
            (
                self.timestamp.timestamp(),
                &self.workout,
                self.total_elapsed_time,
                self.active_time,
                self.total_calories,
                self.metabolic_calories,
                self.avg_heart_rate,
                self.max_heart_rate,
            ),
        )
        .map_err(DatabaseError::Insert)
        .map(|_| ())?;

        for (exercise, series) in &self.series {
            exercise.insert(tx)?;
            for serie in series {
                serie.insert(tx)?;
            }
        }

        Ok(())
    }

    #[allow(clippy::field_reassign_with_default)]
    pub(crate) fn load_from_file<P>(path: P) -> errors::Result<Session>
    where
        P: AsRef<Path>,
    {
        let mut fp = File::open(path.as_ref()).unwrap();
        let entries = fitparser::from_reader(&mut fp).unwrap();

        #[cfg(debug_assertions)]
        {
            let mut txt = vec![];
            entries.iter().for_each(|e| {
                txt.push(format!("{:#?}", e));
            });
            fs::write("activity.txt", txt.join("\n")).unwrap();
        }

        let session_entry = entries
            .iter()
            .find(|r| r.kind() == profile::MesgNum::Session);

        if session_entry.is_none() {
            return Err(ParseFitFileError::SessionEntry());
        }

        let session_entry = session_entry.unwrap();

        let mut session = Session::default();
        session.workout = Self::get_workout_name(&entries)?;
        session_entry.fields().iter().for_each(|f| match f.name() {
            "timestamp" => {
                if let Value::Timestamp(val) = f.value() {
                    session.timestamp = *val;
                }
            }
            "total_elapsed_time" => {
                if let Value::Float64(val) = f.value() {
                    session.total_elapsed_time = *val;
                }
            }
            "active_time" => {
                if let Value::Float64(val) = f.value() {
                    session.active_time = *val;
                }
            }
            "total_calories" => {
                if let Value::UInt16(val) = f.value() {
                    session.total_calories = *val;
                }
            }
            "metabolic_calories" => {
                if let Value::UInt16(val) = f.value() {
                    session.metabolic_calories = *val;
                }
            }
            "avg_heart_rate" => {
                if let Value::UInt8(val) = f.value() {
                    session.avg_heart_rate = *val;
                }
            }
            "max_heart_rate" => {
                if let Value::UInt8(val) = f.value() {
                    session.max_heart_rate = *val;
                }
            }
            _ => (),
        });

        session.series = Serie::get_sets(&entries, &session)?;

        Ok(session)
    }

    fn get_workout_name(entries: &[FitDataRecord]) -> errors::Result<String> {
        let wkt_entry = entries
            .iter()
            .find(|r| r.kind() == profile::MesgNum::Workout);

        if wkt_entry.is_none() {
            return Err(ParseFitFileError::WorkoutEntry());
        }

        let wkt_entry = wkt_entry.unwrap();

        let name = wkt_entry.fields().iter().find(|f| f.name() == "wkt_name");

        if name.is_none() {
            return Err(ParseFitFileError::WorkoutName());
        }

        let name = name.unwrap();

        if let Value::String(name) = name.value() {
            return Ok(name.clone());
        }
        panic!("Invalid workout name type");
    }

    pub fn get_volume(&self) -> f64 {
        let mut accum = 0.0;

        for (_, series) in &self.series {
            for serie in series {
                accum += serie.reps as f64 * serie.weight;
            }
        }

        accum
    }

    pub fn get_num_series(&self) -> u16 {
        let mut accum = 0;

        for (_, series) in &self.series {
            for serie in series {
                accum += serie.reps;
            }
        }

        accum
    }

    pub fn get_density(&self) -> f64 {
        self.get_volume() / self.get_num_series() as f64
    }
}
