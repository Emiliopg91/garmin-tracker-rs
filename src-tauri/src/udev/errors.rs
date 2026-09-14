use thiserror::Error;

#[derive(Error, Debug)]
pub enum UdevError {
    #[error("Error writting udev rules: {0}")]
    ErrorWrittingRules(#[source] std::io::Error),
    #[error("Error reloading udev rules: {0}")]
    ErrorReloadingRules(#[source] std::io::Error),
    #[error("Error triggering udev rules: {0}")]
    ErrorTriggeringRules(#[source] std::io::Error),
}

pub type Result<T> = std::result::Result<T, UdevError>;
