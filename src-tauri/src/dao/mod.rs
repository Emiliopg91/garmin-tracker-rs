pub mod additional_data;
pub mod body_metric;
pub mod device;
pub mod exercise;
pub mod exercise_category;
pub mod lap;
pub mod session;
pub mod set;
pub mod settings;
pub mod sport;
pub mod sub_sport;
pub mod workout;

use rusqlite_orm::{dao::Repository, database::DatabasePool, errors::Result};

use crate::dao::{
    additional_data::AdditionalDataRepository, body_metric::BodyMetricRepository,
    device::DeviceRepository, exercise::ExerciseRepository,
    exercise_category::ExerciseCategoryRepository, lap::LapRepository, session::SessionRepository,
    set::SetRepository, settings::SettingsRepository, sport::SportRepository,
    sub_sport::SubSportRepository, workout::WorkoutRepository,
};

pub fn validate_schemas(database: &DatabasePool) -> Result<()> {
    database.run_in_connection(|conn| {
        AdditionalDataRepository::select().fetch_one_in(conn)?;
        BodyMetricRepository::select().fetch_one_in(conn)?;
        DeviceRepository::select().fetch_one_in(conn)?;
        ExerciseCategoryRepository::select().fetch_one_in(conn)?;
        ExerciseRepository::select().fetch_one_in(conn)?;
        LapRepository::select().fetch_one_in(conn)?;
        SessionRepository::select().fetch_one_in(conn)?;
        SetRepository::select().fetch_one_in(conn)?;
        SettingsRepository::select().fetch_one_in(conn)?;
        SportRepository::select().fetch_one_in(conn)?;
        SubSportRepository::select().fetch_one_in(conn)?;
        WorkoutRepository::select().fetch_one_in(conn)?;

        Ok(())
    })
}
