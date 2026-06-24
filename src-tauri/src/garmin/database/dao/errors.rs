use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseFitFileError {
    #[error("Invalid file format")]
    InvalidFileFormat(),
    #[error("Ony strength training supported")]
    OnlyStrengthTraining(),
}

pub type Result<T> = std::result::Result<T, ParseFitFileError>;
