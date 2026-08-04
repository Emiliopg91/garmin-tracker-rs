use serde::Serialize;

use crate::{dao::session::Session, utils::date_time_utils::DateTimeUtils};

#[derive(Serialize)]
pub struct WorkoutListItem {
    pub name: String,
    pub latest_session: String,
    pub sessions: u32,
    pub avg_time: String,
}

#[derive(Serialize)]
pub struct WorkoutSession {
    pub date: String,
    pub volume: f64,
    pub time: String,
    pub vol_diff: String,
}

impl From<&Session> for WorkoutSession {
    fn from(value: &Session) -> Self {
        let mut volume = 0_f64;

        for serie in &value.series {
            volume += (serie.reps as f64) * serie.weight
        }

        WorkoutSession {
            date: DateTimeUtils::format_time_date(value.date),
            volume,
            time: DateTimeUtils::format_duration(value.total_elapsed_time as u64),
            vol_diff: "-".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct WorkoutDetails {
    pub name: String,
    pub latest_session: String,
    pub session_count: u32,
    pub avg_time: String,
    pub avg_volume: f64,
    pub sessions: Vec<WorkoutSession>,
}
