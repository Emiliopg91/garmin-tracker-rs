use std::{
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use mtp_rs::{MtpDevice, ObjectInfo, Storage};
use tauri_plugin_log::log::{debug, error, info};
use tokio::sync::Mutex;

use crate::{
    dto::devices::DeviceListItem,
    mtp::errors::{MtpError, Result},
};
pub mod errors;

pub static MTP_CLIENT_INST: LazyLock<Mutex<MtpClient>> = LazyLock::new(|| Mutex::new(MtpClient {}));

pub struct MtpClient {}

impl MtpClient {
    pub async fn get_connected_devices(&self) -> Result<Vec<DeviceListItem>> {
        let mut res = Vec::new();

        let devices = MtpDevice::list_devices().map_err(MtpError::ListDevices)?;

        for device_info in devices {
            let device = MtpDevice::open_by_location(device_info.location_id)
                .await
                .map_err(|e| MtpError::OpenDevice(device_info.location_id, e))?;

            let info = device.device_info();
            if info.manufacturer.to_uppercase() == "GARMIN" {
                res.push(DeviceListItem::from(info))
            }

            let _ = device.close().await;
        }

        Ok(res)
    }

    pub async fn download_activities_since(
        &self,
        serial: &str,
        date: String,
    ) -> Result<Vec<PathBuf>> {
        let mut result = Vec::new();

        let devices_info = MtpDevice::list_devices().map_err(MtpError::ListDevices)?;
        if let Some(device_info) = devices_info.iter().find(|d| {
            d.serial_number
                .as_ref()
                .is_some_and(|serial_n| serial_n == serial)
        }) {
            let device = MtpDevice::open_by_location(device_info.location_id)
                .await
                .map_err(|e| MtpError::OpenDevice(device_info.location_id, e))?;
            info!(
                "Found device {} {} with S/N {}",
                device.device_info().manufacturer,
                device.device_info().model,
                serial
            );

            debug!("Entering into GARMIN folder...");
            let storage = &device.storages().await.map_err(MtpError::Storage)?[0];
            if let Some(garmin_folder) = storage
                .list_objects(None)
                .await
                .map_err(MtpError::ListFiles)?
                .iter()
                .find(|oi| oi.filename == "GARMIN")
            {
                debug!("Entering into GARMIN/Activity folder...");
                if let Some(activity_folder) = storage
                    .list_objects(Some(garmin_folder.handle))
                    .await
                    .map_err(MtpError::ListFiles)?
                    .iter()
                    .find(|oi| oi.filename == "Activity")
                {
                    info!("Listing files...");
                    let mut objs = storage
                        .list_objects(Some(activity_folder.handle))
                        .await
                        .map_err(MtpError::ListFiles)?;

                    info!("Found {} files", objs.len());
                    objs.retain(|f| f.filename.split('.').next().unwrap() > date.as_str());

                    if objs.is_empty() {
                        info!("No pending files to import");
                        Ok(result)
                    } else {
                        info!("Pending {} files", objs.len());
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        let tmp_dir =
                            PathBuf::from(format!("/tmp/garmin-tracker-rs-{}", now.as_millis()));

                        if let Err(e) = fs::create_dir_all(&tmp_dir) {
                            Err(MtpError::ErrorCreatingDownloadFolder(
                                tmp_dir.display().to_string(),
                                e,
                            ))
                        } else {
                            info!("Downloading files...");
                            if let Ok(paths) = download_files(&objs, &tmp_dir, storage).await {
                                result = paths;
                            }
                            let _ = device.close().await;

                            info!("Files downloaded");
                            Ok(result)
                        }
                    }
                } else {
                    let _ = device.close().await;
                    Err(MtpError::NoStorageDevice(serial.to_string()))
                }
            } else {
                let _ = device.close().await;
                Err(MtpError::NoStorageDevice(serial.to_string()))
            }
        } else {
            Err(MtpError::MissingDevice(serial.to_string()))
        }
    }
}

async fn download_files(
    objs: &[ObjectInfo],
    tmp_dir: &Path,
    storage: &Storage,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for obj in objs {
        match storage.download(obj.handle, mtp_rs::ByteRange::Full).await {
            Ok(mut data) => {
                let path = tmp_dir.join(&obj.filename);
                let mut bytes = Vec::with_capacity(data.bytes_received() as usize);
                while let Some(window) = data.next_chunk().await {
                    match window {
                        Ok(rec_bytes) => {
                            bytes.extend_from_slice(&rec_bytes);
                        }
                        Err(e) => {
                            error!("Error downloading file {}: {}", obj.filename, e)
                        }
                    }
                }
                fs::write(&path, bytes)
                    .map_err(|e| MtpError::WriteData(path.display().to_string(), e))?;
                paths.push(path);
            }
            Err(e) => {
                error!("Error downloading file {}: {}", obj.filename, e)
            }
        }
    }

    Ok(paths)
}
