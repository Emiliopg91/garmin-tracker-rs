use rusqlite_orm::dao::Repository;
use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod settings_keys {
    pub const AUTO_SYNC: &str = "auto_sync";
    pub const DISTANCE_UNIT: &str = "distance_unit";
    pub const START_ON_BOOT: &str = "start_boot";
    pub const WEIGHT_UNIT: &str = "weight_unit";
}

#[derive(Entity, Serialize, Deserialize)]
#[primary_key(name)]
pub struct Settings {
    pub name: String,
    pub value: String,
}

impl Settings {
    pub fn get_weight_unit() -> WeightUnit {
        SettingsRepository::select_by_id(settings_keys::WEIGHT_UNIT)
            .ok()
            .flatten()
            .and_then(|r| WeightUnit::try_from(r.value.as_str()).ok())
            .unwrap_or(WeightUnit::Kilograms)
    }
    pub fn set_weight_unit(value: &WeightUnit) -> rusqlite_orm::database::errors::Result<()> {
        SettingsRepository::insert()
            .or_replace()
            .item(Settings {
                name: settings_keys::WEIGHT_UNIT.to_string(),
                value: value.to_string(),
            })
            .execute()
            .map(|_| ())
    }

    pub fn get_distance_unit() -> DistanceUnit {
        SettingsRepository::select_by_id(settings_keys::DISTANCE_UNIT)
            .ok()
            .flatten()
            .and_then(|r| DistanceUnit::try_from(r.value.as_str()).ok())
            .unwrap_or(DistanceUnit::Kilometers)
    }
    pub fn set_distance_unit(value: &DistanceUnit) -> rusqlite_orm::database::errors::Result<()> {
        SettingsRepository::insert()
            .or_replace()
            .item(Settings {
                name: settings_keys::DISTANCE_UNIT.to_string(),
                value: value.to_string(),
            })
            .execute()
            .map(|_| ())
    }

    pub fn get_auto_sync() -> bool {
        SettingsRepository::select_by_id(settings_keys::AUTO_SYNC)
            .ok()
            .flatten()
            .and_then(|r| r.value.parse().ok())
            .unwrap_or(true)
    }
    pub fn set_auto_sync(value: bool) -> rusqlite_orm::database::errors::Result<()> {
        SettingsRepository::insert()
            .or_replace()
            .item(Settings {
                name: settings_keys::AUTO_SYNC.to_string(),
                value: value.to_string(),
            })
            .execute()
            .map(|_| ())
    }

    pub fn get_start_on_boot() -> bool {
        SettingsRepository::select_by_id(settings_keys::START_ON_BOOT)
            .ok()
            .flatten()
            .and_then(|r| r.value.parse().ok())
            .unwrap_or(false)
    }
    pub fn set_start_on_boot(value: bool) -> rusqlite_orm::database::errors::Result<()> {
        SettingsRepository::insert()
            .or_replace()
            .item(Settings {
                name: settings_keys::START_ON_BOOT.to_string(),
                value: value.to_string(),
            })
            .execute()
            .map(|_| ())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistanceUnit {
    Kilometers,
    Miles,
}

impl TryFrom<&str> for DistanceUnit {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Kilometers" => Ok(DistanceUnit::Kilometers),
            "Miles" => Ok(DistanceUnit::Miles),
            other => Err(format!("Invalid DistanceUnit '{other}'")),
        }
    }
}

impl fmt::Display for DistanceUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DistanceUnit::Kilometers => "Kilometers",
            DistanceUnit::Miles => "Miles",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeightUnit {
    Kilograms,
    Pounds,
}

impl TryFrom<&str> for WeightUnit {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Kilograms" => Ok(WeightUnit::Kilograms),
            "Pounds" => Ok(WeightUnit::Pounds),
            other => Err(format!("Invalid WeightUnit '{other}'")),
        }
    }
}

impl fmt::Display for WeightUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WeightUnit::Kilograms => "Kilograms",
            WeightUnit::Pounds => "Pounds",
        };
        write!(f, "{s}")
    }
}
