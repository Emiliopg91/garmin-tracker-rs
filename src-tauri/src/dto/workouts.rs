use serde::Serialize;

use crate::dao::session::Session;

#[derive(Serialize)]
pub struct WorkoutListItem {
    pub name: String,
    pub latest_session: i32,
    pub sessions: u32,
    pub avg_time: i32,
}

#[derive(Serialize)]
pub struct WorkoutSession {
    pub date: i32,
    pub volume: f64,
    pub time: i32,
    pub vol_diff: String,
}

impl From<&Session> for WorkoutSession {
    fn from(value: &Session) -> Self {
        WorkoutSession {
            date: value.date as i32,
            volume: 0_f64,
            time: value.total_elapsed_time.round() as i32,
            vol_diff: "-".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct WorkoutDetails {
    pub name: String,
    pub latest_session: i32,
    pub session_count: u32,
    pub avg_time: i32,
    pub avg_volume: f64,
    pub sessions: Vec<WorkoutSession>,
}
