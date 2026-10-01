use thiserror::Error;

#[derive(Error, Debug)]
pub enum WriteFitFileError {
    #[error("Error while creating file {0}: {1}")]
    FileCreating(String, #[source] std::io::Error),
    #[error("Error while writing file {0}: {1}")]
    FileWriting(String, #[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Missing {0} field")]
    MissingField(String),
}

pub type Result<T> = std::result::Result<T, WriteFitFileError>;
