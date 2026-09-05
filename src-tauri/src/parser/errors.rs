use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseFitFileError {
    #[error("Error while opening file {0}: {1}")]
    FileOpening(String, #[source] std::io::Error),
    #[error("Error while reading file {0}: {1}")]
    FileReading(String, #[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Missing {0} field")]
    MissingField(String),
    #[error("Unknown exercise (category={0}, id={1})")]
    UnknownExercise(u16, u16),
}

pub type Result<T> = std::result::Result<T, ParseFitFileError>;
