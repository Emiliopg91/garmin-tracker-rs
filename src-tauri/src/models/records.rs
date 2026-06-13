use serde::Serialize;

#[derive(Serialize)]
pub struct RecordListItem {
    pub exercise: String,
    pub reps: u16,
    pub weight: f64,
    pub rm: f64,
}
