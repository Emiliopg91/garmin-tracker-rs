use std::fs;

use serde::{Deserialize, Serialize};

use crate::utils::constants;

#[derive(Clone, Serialize, Deserialize)]
pub enum DistanceUnit {
    Meter,
    Mile,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum WeightUnit {
    Kilogram,
    Pounds,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub distance_unit: DistanceUnit,
    pub weight_unit: WeightUnit,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            distance_unit: DistanceUnit::Meter,
            weight_unit: WeightUnit::Kilogram,
        }
    }
}

impl Settings {
    pub fn initialize() -> Result<Self, Box<dyn std::error::Error>> {
        let mut result = Self::default();

        let path = (*constants::SETTINGS_FILE).clone();
        if fs::exists(&path).unwrap() {
            let content = fs::read_to_string(&path).unwrap();
            result = serde_yaml::from_str(&content).unwrap();
        }

        Ok(result)
    }
}
