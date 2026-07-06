use mtp_rs::DeviceInfo;
use serde::Serialize;

use crate::garmin::database::dao::device::Device;

#[derive(Serialize, Clone)]
pub struct DeviceListItem {
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
}

impl From<&DeviceInfo> for DeviceListItem {
    fn from(value: &DeviceInfo) -> Self {
        Self {
            manufacturer: value.manufacturer.clone(),
            model: value.model.clone(),
            serial_number: value.serial_number.clone(),
        }
    }
}

impl From<&DeviceListItem> for Device {
    fn from(value: &DeviceListItem) -> Self {
        Device {
            serial: value.serial_number.clone(),
            model: value.model.clone(),
            last_sync: None,
        }
    }
}
