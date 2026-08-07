use garmin_tracker_rs_macros::translate;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseFitFileError {
    #[error("{}", translate!("error_parser_file_opening", .0, .1))]
    FileOpening(String, #[source] std::io::Error),
    #[error("{}", translate!("error_parser_file_reading", .0, .1))]
    FileReading(String, #[source] fitparser::Error),
    #[error("{}", translate!("error_parser_missing_field", .0))]
    MissingField(String),
    #[error("{}", translate!("error_parser_invalid_field_value", .0, .1))]
    InvalidFieldValue(String, String),
    #[error("{0}")]
    GenericError(String),
}

pub type Result<T> = std::result::Result<T, ParseFitFileError>;
