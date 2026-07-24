use chrono::{Datelike, Local, TimeZone, Timelike};
use garmin_tracker_rs_macros::Entity;
use indexmap::IndexMap;

use crate::garmin::database::dao::{
    Entity,
    helpers::types::{order_by::OrderBy, where_clause::Where},
};

use super::{exercise::Exercise, serie::Serie};

#[derive(Default, Debug, Entity, Clone)]
pub struct Session {
    #[id]
    pub date: i64,

    pub workout: String,

    pub total_elapsed_time: f64,
    pub active_time: f64,

    pub total_calories: u16,
    pub metabolic_calories: u16,

    pub avg_heart_rate: u8,
    pub max_heart_rate: u8,

    pub training_load: f64,

    pub sub_sport: String,

    #[no_column]
    pub series: IndexMap<Exercise, Vec<Serie>>,
}
impl Session {
    pub fn format_date(&self) -> String {
        let timestamp = Local.timestamp_opt(self.date, 0).unwrap();
        format!(
            "{:02}:{:02} {:02}/{:02}/{:04}",
            timestamp.hour(),
            timestamp.minute(),
            timestamp.day(),
            timestamp.month(),
            timestamp.year()
        )
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

    pub fn find_by_id(
        timestamp: i64,
        with_series: bool,
    ) -> crate::garmin::database::errors::Result<Option<Session>> {
        let opt_sess = Session::select_by_id(timestamp)?;

        Ok(match opt_sess {
            Some(mut session) => {
                if with_series {
                    session.series = if session.sub_sport == "strength_training" {
                        Serie::load_for_session(session.date)?
                    } else {
                        IndexMap::new()
                    };
                }
                Some(session)
            }
            None => None,
        })
    }

    pub fn find_by_workout(workout: &str) -> crate::garmin::database::errors::Result<Vec<Session>> {
        let mut res = Session::select()
            .where_(Where::Eq(SESSION_COLUMN_WORKOUT, workout.into()))
            .order_by(OrderBy::Desc(SESSION_COLUMN_DATE))
            .fetch()?;

        for r in &mut res {
            r.series = Serie::load_for_session(r.date)?;
        }

        Ok(res)
    }

    pub fn load_from_db(
        with_series: bool,
    ) -> crate::garmin::database::errors::Result<Vec<Session>> {
        let mut res = Session::select()
            .order_by(OrderBy::Desc(SESSION_COLUMN_DATE))
            .fetch()?;

        if with_series {
            for r in &mut res {
                r.series = if r.sub_sport == "strength_training" {
                    Serie::load_for_session(r.date)?
                } else {
                    IndexMap::new()
                };
            }
        }

        Ok(res)
    }
}
