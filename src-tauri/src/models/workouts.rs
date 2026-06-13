use serde::Serialize;

#[derive(Serialize)]
pub struct WorkoutListItem {
    pub name: String,
    pub date: String,
    pub timestamp: i64,
}
