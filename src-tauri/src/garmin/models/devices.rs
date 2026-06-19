use mtp_rs::ptp::DeviceInfo;
use serde::Serialize;

#[derive(Serialize)]
pub struct DeviceListItem {
    manufacturer: String,
    model: String,
    serial_number: String,
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
