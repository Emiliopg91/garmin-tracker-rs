use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseFitFileError {
    #[error("Missing session entry")]
    SessionEntry(),
    #[error("Missing workout entry")]
    WorkoutEntry(),
    #[error("Missing workout name")]
    WorkoutName(),
    #[error("Invalid workout name")]
    InvalidWorkoutName(),
    #[error("Missing exercise name")]
    ExerciseName(),
    #[error("Missing exercise category")]
    ExerciseCategory(),
}

pub type Result<T> = std::result::Result<T, ParseFitFileError>;
