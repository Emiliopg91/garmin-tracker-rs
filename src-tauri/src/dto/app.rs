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
    pub start_boot: bool,
}
