use serde::Serialize;

#[derive(Serialize)]
pub struct ExerciseListItem {
    pub category: String,
    pub id: u16,
    pub name: String,
}
