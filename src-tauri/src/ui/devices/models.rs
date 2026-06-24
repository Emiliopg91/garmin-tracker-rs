use mtp_rs::ptp::DeviceInfo;
use serde::Serialize;

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
