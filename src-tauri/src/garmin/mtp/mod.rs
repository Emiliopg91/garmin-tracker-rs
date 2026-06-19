use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mtp_rs::{MtpDevice, ptp::ObjectInfo};

use crate::garmin::{
    models::devices::DeviceListItem,
    mtp::errors::{MtpError, Result},
};

pub mod errors;

pub struct MtpClient {}

impl MtpClient {
    pub async fn get_connected_devices() -> Result<Vec<DeviceListItem>> {
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

    pub async fn download_activities_since(serial: &str, date: String) -> Result<Vec<PathBuf>> {
        let mut result = Vec::new();

        let devices_info = MtpDevice::list_devices().map_err(MtpError::ListDevices)?;
        let device_info = devices_info
            .iter()
            .find(|d| {
                if let Some(serial_n) = &d.serial_number
                    && serial_n == serial
                {
                    true
                } else {
                    false
                }
            })
            .unwrap();

        let device = MtpDevice::open_by_location(device_info.location_id)
            .await
            .map_err(|e| MtpError::OpenDevice(device_info.location_id, e))?;

        let storage = &device.storages().await.map_err(MtpError::Storage)?[0];
        if let Some(garmin_folder) = storage
            .list_objects(None)
            .await
            .map_err(MtpError::ListFiles)?
            .iter()
            .find(|oi| oi.filename == "GARMIN")
            && let Some(activity_folder) = storage
                .list_objects(Some(garmin_folder.handle))
                .await
                .map_err(MtpError::ListFiles)?
                .iter()
                .find(|oi| oi.filename == "Activity")
        {
            let mut objs = storage
                .list_objects(Some(activity_folder.handle))
                .await
                .map_err(MtpError::ListFiles)?;

            objs = objs
                .iter()
                .filter(|f| f.filename.split('.').nth(0).unwrap() > date.as_str())
                .cloned()
                .collect::<Vec<ObjectInfo>>();

            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let tmp_dir = PathBuf::from(format!("/tmp/garmin-tracker-rs-{}", now.as_millis()));

            fs::create_dir_all(&tmp_dir).unwrap();

            for obj in objs {
                let data = storage
                    .download(obj.handle)
                    .await
                    .map_err(|e| MtpError::DownloadFile(obj.filename.clone(), e))?;
                let path = tmp_dir.join(obj.filename);
                fs::write(&path, data)
                    .map_err(|e| MtpError::WriteData(path.display().to_string(), e))?;
                result.push(path);
            }
        }

        let _ = device.close().await;

        Ok(result)
    }
}
