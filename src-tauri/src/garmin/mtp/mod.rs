use std::{
    fs,
    path::PathBuf,
    sync::LazyLock,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use mtp_rs::{MtpDevice, ObjectInfo};
use tauri_plugin_log::log::{debug, info};
use tokio::sync::Mutex;

use crate::{
    garmin::mtp::errors::{MtpError, Result},
    ui::devices::models::DeviceListItem,
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
            if let Some(serial_n) = &d.serial_number
                && serial_n == serial
            {
                true
            } else {
                false
            }
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
                    objs = objs
                        .iter()
                        .filter(|f| f.filename.split('.').nth(0).unwrap() > date.as_str())
                        .cloned()
                        .collect::<Vec<ObjectInfo>>();

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
                            for obj in objs {
                                sleep(Duration::from_millis(100));
                                let mut data = storage
                                    .download(obj.handle, mtp_rs::ByteRange::Full)
                                    .await
                                    .map_err(|e| MtpError::DownloadFile(obj.filename.clone(), e))?;
                                let path = tmp_dir.join(obj.filename);
                                let mut bytes = Vec::with_capacity(data.bytes_received() as usize);
                                while let Some(window) = data.next_chunk().await
                                    && let Ok(rec_bytes) = window
                                {
                                    bytes.extend_from_slice(&rec_bytes);
                                }
                                fs::write(&path, bytes).map_err(|e| {
                                    MtpError::WriteData(path.display().to_string(), e)
                                })?;
                                result.push(path);
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
