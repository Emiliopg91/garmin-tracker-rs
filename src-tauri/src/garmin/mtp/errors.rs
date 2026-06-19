use thiserror::Error;

#[derive(Error, Debug)]
pub enum MtpError {
    #[error("Cannot list devices:\n  {0}")]
    ListDevices(mtp_rs::Error),
    #[error("Cannot open device {0}:\n  {1}")]
    OpenDevice(u64, mtp_rs::Error),
    #[error("Cannot access storage:\n  {0}")]
    Storage(mtp_rs::Error),
    #[error("Cannot list files:\n  {0}")]
    ListFiles(mtp_rs::Error),
    #[error("Cannot download file GARMIN/Activity/{0}:\n  {1}")]
    DownloadFile(String, mtp_rs::Error),
    #[error("Cannot write data to {0}:\n  {1}")]
    WriteData(String, std::io::Error),
}

pub type Result<T> = std::result::Result<T, MtpError>;
