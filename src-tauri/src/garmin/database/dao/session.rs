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

    pub training_load: f64,

    pub sub_sport: String,

    pub series: IndexMap<Exercise, Vec<Serie>>,
}
impl Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.workout, self.format_date())
    }
}
impl Session {
    const FIELD_LIST: &str = "date, workout, total_elapsed_time, active_time, total_calories, metabolic_calories, avg_heart_rate, max_heart_rate, training_load, sub_sport";

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

    pub fn find_by_id(timestamp: i64, with_series: bool) -> Result<Option<Session>> {
        let opt_sess = {
            let mut db = DATABASE_INST.lock().unwrap();
            let conn = db.get_connection()?;

            let result = conn.query_row(
                &format!("SELECT {} FROM SESSION WHERE date=?", Self::FIELD_LIST),
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
                if with_series {
                    session.series = Serie::load_for_session(session.timestamp)?;
                }
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
                .prepare(&format!(
                    "SELECT {} FROM SESSION WHERE workout=? ORDER BY date DESC",
                    Self::FIELD_LIST
                ))
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

    pub fn load_from_db(with_series: bool) -> Result<Vec<Session>> {
        let mut res = Vec::new();

        {
            let mut db = DATABASE_INST.lock().unwrap();
            let conn = db.get_connection()?;

            let mut stmt = conn
                .prepare(&format!(
                    "SELECT {} FROM SESSION ORDER BY date DESC",
                    Self::FIELD_LIST
                ))
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

        if with_series {
            for r in &mut res {
                r.series = if r.sub_sport == "strength_training" {
                    Serie::load_for_session(r.timestamp)?
                } else {
                    IndexMap::new()
                };
            }
        }

        Ok(res)
    }

    fn map_from_row(row: &Row) -> std::result::Result<Self, rusqlite::Error> {
        Ok(Session {
            timestamp: Local.timestamp_opt(row.get::<_, i64>("date")?, 0).unwrap(),
            workout: row.get::<_, String>("workout")?,
            total_elapsed_time: row.get::<_, f64>("total_elapsed_time")?,
            active_time: row.get::<_, f64>("active_time")?,
            total_calories: row.get::<_, u16>("total_calories")?,
            metabolic_calories: row.get::<_, u16>("metabolic_calories")?,
            avg_heart_rate: row.get::<_, u8>("avg_heart_rate")?,
            max_heart_rate: row.get::<_, u8>("max_heart_rate")?,
            training_load: row.get::<_, f64>("training_load")?,
            sub_sport: row.get::<_, String>("sub_sport")?,
            series: IndexMap::new(),
        })
    }

    pub fn insert(&mut self) -> crate::garmin::database::errors::Result<()> {
        if let Ok(mut db) = DATABASE_INST.lock() {
            db.run_in_transaction(|tx| {
                tx.execute(
                    &format!(
                        "INSERT INTO SESSION({}) VALUES(?,?,?,?,?,?,?,?,?,?)",
                        Self::FIELD_LIST
                    ),
                    (
                        self.timestamp.timestamp(),
                        &self.workout,
                        self.total_elapsed_time,
                        self.active_time,
                        self.total_calories,
                        self.metabolic_calories,
                        self.avg_heart_rate,
                        self.max_heart_rate,
                        self.training_load,
                        &self.sub_sport,
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
            })?
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
}
