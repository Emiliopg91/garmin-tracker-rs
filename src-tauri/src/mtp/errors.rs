use garmin_tracker_rs_macros::translate;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MtpError {
    #[error("{}", translate!("error_mtp_list_devices", .0))]
    ListDevices(#[source] mtp_rs::Error),
    #[error("{}", translate!("error_mtp_open_device", .0, .1))]
    OpenDevice(u64, #[source] mtp_rs::Error),
    #[error("{}", translate!("error_mtp_storage", .0))]
    Storage(#[source] mtp_rs::Error),
    #[error("{}", translate!("error_mtp_list_files", .0))]
    ListFiles(#[source] mtp_rs::Error),
    #[error("{}", translate!("error_mtp_download_file", .0, .1))]
    DownloadFile(String, #[source] mtp_rs::Error),
    #[error("{}", translate!("error_mtp_write_data", .0, .1))]
    WriteData(String, #[source] std::io::Error),
    #[error("{}", translate!("error_mtp_missing_device", .0))]
    MissingDevice(String),
    #[error("{}", translate!("error_mtp_no_storage_device", .0))]
    NoStorageDevice(String),
    #[error("{}", translate!("error_mtp_creating_download_folder", .0, .1))]
    ErrorCreatingDownloadFolder(String, #[source] std::io::Error),
}

pub type Result<T> = std::result::Result<T, MtpError>;
