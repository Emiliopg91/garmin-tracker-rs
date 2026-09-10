use rusqlite_orm::database::DatabasePool;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub enum AppEnvironment {
    Debug,
    Release,
}

use crate::{
    dao::settings::{DistanceUnit, WeightUnit},
    utils::translations::Languages,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub distance_unit: DistanceUnit,
    pub weight_unit: WeightUnit,
    pub auto_sync: bool,
    pub start_boot: bool,
    pub language: Languages,
}

impl From<&DatabasePool> for Settings {
    fn from(database: &DatabasePool) -> Self {
        Self {
            auto_sync: crate::dao::settings::Settings::get_auto_sync(database),
            distance_unit: crate::dao::settings::Settings::get_distance_unit(database),
            language: crate::dao::settings::Settings::get_language(database),
            start_boot: crate::dao::settings::Settings::get_start_on_boot(database),
            weight_unit: crate::dao::settings::Settings::get_weight_unit(database),
        }
    }
}
