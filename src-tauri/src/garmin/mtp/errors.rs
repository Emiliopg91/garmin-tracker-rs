use thiserror::Error;

#[derive(Error, Debug)]
pub enum MtpError {
    #[error("Cannot list devices {0}")]
    ListDevices(mtp_rs::Error),
    #[error("Cannot open device {0} {1}")]
    OpenDevice(u64, mtp_rs::Error),
    #[error("Cannot access storage {0}")]
    Storage(mtp_rs::Error),
    #[error("Cannot list files {0}")]
    ListFiles(mtp_rs::Error),
    #[error("Cannot download file GARMIN/Activity/{0} {1}")]
    DownloadFile(String, mtp_rs::Error),
    #[error("Cannot write data to {0} {1}")]
    WriteData(String, std::io::Error),
    #[error("Cannot find device with serial number {0}")]
    MissingDevice(String),
    #[error("No storage for device with serial number {0}")]
    NoStorageDevice(String),
    #[error("Error creating download folder {0} {1}")]
    ErrorCreatingDownloadFolder(String, std::io::Error),
}

pub type Result<T> = std::result::Result<T, MtpError>;
