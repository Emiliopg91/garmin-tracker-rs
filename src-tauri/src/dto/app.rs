use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub enum AppEnvironment {
    Debug,
    Release,
}

use crate::dao::settings::{DistanceUnit, WeightUnit};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub distance_unit: DistanceUnit,
    pub weight_unit: WeightUnit,
    pub auto_sync: bool,
}

impl Settings {
    pub fn load() -> Self {
        Self {
            auto_sync: crate::dao::settings::Settings::get_auto_sync(),
            distance_unit: crate::dao::settings::Settings::get_distance_unit(),
            weight_unit: crate::dao::settings::Settings::get_weight_unit(),
        }
    }
}
