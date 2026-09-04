use std::{
    path::PathBuf,
    sync::LazyLock,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use mtp_rs::MtpDevice;
use tauri_plugin_log::log::{debug, error, info};
use tokio::{fs, sync::Mutex};

use crate::{
    dto::devices::DeviceListItem,
    mtp::errors::{MtpError, Result},
    utils::constants,
};
pub mod errors;

pub static MTP_CLIENT_INST: LazyLock<Mutex<MtpClient>> = LazyLock::new(|| Mutex::new(MtpClient {}));

pub struct MtpClient {}

impl MtpClient {
    /// Lists MTP devices currently connected over USB, filtered to Garmin ones.
    pub async fn get_connected_devices(&self) -> Result<Vec<DeviceListItem>> {
        let mut res = Vec::new();

        let devices = MtpDevice::list_devices().map_err(MtpError::ListDevices)?;

        for device_info in devices {
            let device = MtpDevice::open_by_location(device_info.location_id)
                .await
                .map_err(|e| MtpError::OpenDevice(device_info.location_id, e))?;

            let info = device.device_info();
            if info.manufacturer.to_uppercase() == constants::MTP_GARMIN_MANUFACTURER {
                res.push(DeviceListItem::from(info))
            }

            let _ = device.close().await;
        }

        Ok(res)
    }

    /// Downloads `.FIT` activity files newer than `date` from the device's `GARMIN/Activity` folder into a temp directory, returning their local paths.
    pub async fn download_activities_since(
        &self,
        serial: &str,
        date: String,
    ) -> Result<Option<PathBuf>> {
        let devices_info = MtpDevice::list_devices().map_err(MtpError::ListDevices)?;
        let device_info = devices_info
            .iter()
            .find(|d| {
                d.serial_number
                    .as_ref()
                    .is_some_and(|serial_n| serial_n == serial)
            })
            .ok_or_else(|| MtpError::MissingDevice(serial.to_string()))?;

        let device = MtpDevice::open_by_location(device_info.location_id)
            .await
            .map_err(|e| MtpError::OpenDevice(device_info.location_id, e))?;
        info!(
            "Found device {} {} with S/N {}",
            device.device_info().manufacturer,
            device.device_info().model,
            serial
        );

        // Every fallible step below runs inside this block so that, regardless of how it
        // exits, `device.close()` below always runs exactly once.
        let result: Result<Option<PathBuf>> = async {
            debug!("Entering into GARMIN folder...");
            let storages = device.storages().await.map_err(MtpError::Storage)?;
            let storage = storages
                .first()
                .ok_or_else(|| MtpError::NoStorageDevice(serial.to_string()))?;

            let garmin_folder = storage
                .list_objects(None)
                .await
                .map_err(MtpError::ListFiles)?
                .into_iter()
                .find(|oi| oi.filename == constants::MTP_GARMIN_ROOT_FOLDER)
                .ok_or_else(|| MtpError::NoStorageDevice(serial.to_string()))?;

            debug!("Entering into GARMIN/Activity folder...");
            let activity_folder = storage
                .list_objects(Some(garmin_folder.handle))
                .await
                .map_err(MtpError::ListFiles)?
                .into_iter()
                .find(|oi| oi.filename == constants::MTP_GARMIN_ACTIVITY_FOLDER)
                .ok_or_else(|| MtpError::NoStorageDevice(serial.to_string()))?;

            info!("Listing files...");
            let mut objs = storage
                .list_objects(Some(activity_folder.handle))
                .await
                .map_err(MtpError::ListFiles)?;

            info!("Found {} files", objs.len());
            objs.retain(|f| f.filename.split('.').next().unwrap() > date.as_str());

            if objs.is_empty() {
                info!("No pending files to import");
                return Ok(None);
            }

            info!("Pending {} files", objs.len());
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let tmp_dir = std::env::temp_dir()
                .join(format!("{}-{}", constants::MTP_TMP_DIR_PREFIX, now.as_millis()));

            fs::create_dir_all(&tmp_dir).await.map_err(|e| {
                MtpError::ErrorCreatingDownloadFolder(tmp_dir.display().to_string(), e)
            })?;

            info!("Downloading files...");
            let t0 = Instant::now();
            let mut size = 0_f64;
            let mut counter = 0_i64;
            for obj in objs {
                match storage.download_to_vec(obj.handle).await {
                    Ok(bytes) => {
                        let path = tmp_dir.join(&obj.filename);
                        fs::write(&path, &bytes)
                            .await
                            .map_err(|e| MtpError::WriteData(path.display().to_string(), e))?;
                        size += bytes.len() as f64;
                        counter += 1;
                    }
                    Err(e) => {
                        error!("Error downloading file {}: {}", obj.filename, e)
                    }
                }
            }

            let elapsed_secs = t0.elapsed().as_secs_f64();
            info!(
                "{} files downloaded in {:.3}s ({:.2} MB/s)",
                counter,
                elapsed_secs,
                (size / (1024 * 1024) as f64) / elapsed_secs
            );
            Ok(Some(tmp_dir))
        }
        .await;

        let _ = device.close().await;
        result
    }
}
