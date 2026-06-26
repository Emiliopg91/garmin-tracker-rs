use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseFitFileError {
    #[error("Error while opening file {0}:\n  {1}")]
    FileOpening(String, std::io::Error),
    #[error("Error while reading file {0}:\n  {1}")]
    FileReading(String, fitparser::Error),
    #[error("Missing {0} field")]
    MissingField(String),
    #[error("Invalid {0} field format: expected {1}")]
    InvalidFieldValue(String, String),
    #[error("Only strength training supported")]
    OnlyStrengthTraining(),
    #[error("{0}")]
    GenericError(String),
}

pub type Result<T> = std::result::Result<T, ParseFitFileError>;
