use serde::Serialize;

use crate::garmin::database::dao::session::Session;

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
}

impl From<&Session> for WorkoutSession {
    fn from(value: &Session) -> Self {
        WorkoutSession {
            date: value.format_date(),
            volume: value.get_volume(),
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
