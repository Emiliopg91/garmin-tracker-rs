use std::fmt::Display;

use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use indexmap::IndexMap;
use rusqlite::Row;

use crate::garmin::database::{
    DATABASE_INST,
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
            "{:02}:{:02} {:02}/{:02}/{:04}",
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

    pub fn format_duration(seconds: u64) -> String {
        let h = seconds / 3600;
        let m = (seconds % 3600) / 60;
        let s = seconds % 60;

        let mut res = if h > 0 {
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else if m > 0 {
            format!("{:02}:{:02}", m, s)
        } else {
            format!("{s}")
        };

        while res.starts_with("0") {
            res.remove(0);
        }

        res
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

    pub fn find_latest() -> Result<Option<Session>> {
        let opt_sess = {
            let mut db = DATABASE_INST.lock().unwrap();
            let conn = db.get_connection()?;

            let result = conn.query_row(
                "SELECT * FROM SESSION ORDER BY DATE DESC LIMIT 1",
                [],
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

    pub fn find_by_workout(workout: &str) -> Result<Vec<Session>> {
        let mut res = Vec::new();

        {
            let mut db = DATABASE_INST.lock().unwrap();
            let conn = db.get_connection()?;

            let mut stmt = conn
                .prepare("SELECT * FROM SESSION WHERE workout=? ORDER BY date DESC")
                .map_err(DatabaseError::Select)?;

            let rows = stmt
                .query_map([workout], Self::map_from_row)
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
        &mut self,
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

        for (exercise, series) in &mut self.series {
            exercise.insert(tx)?;
            for serie in series.iter_mut() {
                serie.insert(tx)?;
            }
            Serie::update_pr(tx, &exercise.category.to_string(), exercise.id);
        }

        Ok(())
    }

    pub fn get_volume(&self) -> f64 {
        let mut volume = 0_f64;

        for (_, series) in &self.series {
            for serie in series {
                volume += (serie.reps as f64) * serie.weight
            }
        }

        volume
    }

    pub fn get_exercises_num(&self) -> u8 {
        let mut exercises = 0_u8;

        for (_, _) in &self.series {
            exercises += 1;
        }

        exercises
    }

    pub fn get_series_num(&self) -> u8 {
        let mut series = 0_u8;

        for (_, series_arr) in &self.series {
            series += series_arr.len() as u8;
        }

        series
    }
}
