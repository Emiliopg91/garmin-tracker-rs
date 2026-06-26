use std::{fs::File, path::Path};

use chrono::{DateTime, Local};
use fitparser::{FitDataField, FitDataRecord, Value, profile};
use indexmap::IndexMap;

use self::errors::ParseFitFileError;

use super::database::dao::{exercise::Exercise, serie::Serie, session::Session};

pub mod errors;

#[allow(clippy::field_reassign_with_default)]
pub(crate) fn load_from_file<P>(path: P) -> errors::Result<Session>
where
    P: AsRef<Path>,
{
    let mut fp = File::open(path.as_ref())
        .map_err(|e| ParseFitFileError::FileOpening(path.as_ref().display().to_string(), e))?;
    let entries = fitparser::from_reader(&mut fp)
        .map_err(|e| ParseFitFileError::FileReading(path.as_ref().display().to_string(), e))?
        .iter()
        .filter(|r| match r.kind() {
            profile::MesgNum::ExerciseTitle
            | profile::MesgNum::Session
            | profile::MesgNum::Workout
            | profile::MesgNum::WorkoutStep
            | profile::MesgNum::Set => true,
            _ => false,
        })
        .cloned()
        .collect::<Vec<FitDataRecord>>();

    #[cfg(debug_assertions)]
    {
        use std::fs;
        let _ = fs::write(
            "/var/mnt/Datos/Desarrollo/Workspace/VSCode/taurfit/activity.txt",
            format!("{:#?}", entries),
        );
    }

    if let Some(session_entry) = entries
        .iter()
        .find(|r| r.kind() == profile::MesgNum::Session)
    {
        if let Some(sub_sport) = session_entry
            .fields()
            .iter()
            .find(|s| s.name() == "sub_sport")
        {
            if let Value::String(subsport_name) = sub_sport.value()
                && subsport_name == "strength_training"
            {
                match get_timestamp("timestamp", session_entry.fields()) {
                    Ok(timestamp) => {
                        let workout = get_workout_name(&entries)?;
                        let total_elapsed_time =
                            get_f64("total_elapsed_time", session_entry.fields()).unwrap_or(0_f64);
                        let active_time =
                            get_f64("active_time", session_entry.fields()).unwrap_or(0_f64);
                        let total_calories =
                            get_u16("total_calories", session_entry.fields()).unwrap_or(0);
                        let metabolic_calories =
                            get_u16("metabolic_calories", session_entry.fields()).unwrap_or(0);
                        let avg_heart_rate =
                            get_u8("avg_heart_rate", session_entry.fields()).unwrap_or(0);
                        let max_heart_rate =
                            get_u8("max_heart_rate", session_entry.fields()).unwrap_or(0);
                        let series = get_sets(&entries, &timestamp)?;

                        Ok(Session {
                            workout,
                            timestamp,
                            total_elapsed_time,
                            active_time,
                            total_calories,
                            metabolic_calories,
                            avg_heart_rate,
                            max_heart_rate,
                            series,
                        })
                    }
                    Err(e) => Err(e),
                }
            } else {
                Err(ParseFitFileError::OnlyStrengthTraining())
            }
        } else {
            Err(ParseFitFileError::MissingField("sub_sport".to_string()))
        }
    } else {
        Err(ParseFitFileError::MissingField("session".to_string()))
    }
}

fn get_workout_name(entries: &[FitDataRecord]) -> errors::Result<String> {
    if let Some(wkt_entry) = entries
        .iter()
        .find(|r| r.kind() == profile::MesgNum::Workout)
    {
        if let Ok(name) = get_string("wkt_name", wkt_entry.fields()) {
            Ok(name)
        } else {
            Err(ParseFitFileError::InvalidFieldValue(
                "name".to_string(),
                "string".to_string(),
            ))
        }
    } else {
        Err(ParseFitFileError::MissingField("workout".to_string()))
    }
}

fn get_sets(
    entries: &[FitDataRecord],
    timestamp: &DateTime<Local>,
) -> errors::Result<IndexMap<Exercise, Vec<Serie>>> {
    let exercises = get_exercises(entries)?;

    let steps = get_steps(entries, &exercises)?;

    let mut sets = IndexMap::<Exercise, Vec<Serie>>::new();
    let mut idx = 0_u8;
    for reg in entries.iter().filter(|r| r.kind() == profile::MesgNum::Set) {
        if let Ok(reps) = get_u16("repetitions", reg.fields())
            && let Ok(weight) = get_f64("weight", reg.fields())
            && let Ok(ex_idx) = get_i64("wkt_step_index", reg.fields())
            && let Some(exercise) = steps.get(ex_idx as usize)
            && let Some(exercise) = exercise
        {
            sets.entry(exercise.clone()).or_default().push(Serie {
                session: *timestamp,
                idx,
                exercise_category: exercise.category.clone(),
                exercise_id: exercise.id,
                reps,
                weight,
                pr: false,
            });
            idx += 1;
        }
    }

    Ok(sets)
}

fn get_steps(
    entries: &[FitDataRecord],
    exercises: &[Exercise],
) -> errors::Result<Vec<Option<Exercise>>> {
    entries
        .iter()
        .filter(|r| r.kind() == profile::MesgNum::WorkoutStep)
        .map(|reg| {
            let Ok(ex_cat) = get_string("exercise_category", reg.fields()) else {
                return Ok(None);
            };

            let ex_id = get_u16("exercise_name", reg.fields()).unwrap_or(1);

            exercises
                .iter()
                .find(|e| e.id == ex_id && e.category == *ex_cat)
                .cloned()
                .ok_or_else(|| {
                    ParseFitFileError::GenericError(format!(
                        "Unknown exercise with category {} and id {}",
                        ex_cat, ex_id
                    ))
                })
                .map(Some)
        })
        .collect()
}

pub fn get_exercises(entries: &[FitDataRecord]) -> errors::Result<Vec<Exercise>> {
    entries
        .iter()
        .filter(|r| r.kind() == profile::MesgNum::ExerciseTitle)
        .map(|reg| {
            Ok(Exercise {
                id: get_u16("exercise_name", reg.fields()).unwrap_or(1),
                category: get_string("exercise_category", reg.fields())?,
                name: get_string("wkt_step_name", reg.fields())?,
            })
        })
        .collect()
}

fn get_f64(name: &str, entries: &[FitDataField]) -> errors::Result<f64> {
    match get_field(name, entries) {
        Ok(Value::Float64(v)) => Ok(*v),
        Ok(_) => Err(ParseFitFileError::InvalidFieldValue(
            name.to_string(),
            "f64".to_string(),
        )),
        Err(e) => Err(e),
    }
}

fn get_u16(name: &str, entries: &[FitDataField]) -> errors::Result<u16> {
    match get_field(name, entries) {
        Ok(Value::UInt16(v)) => Ok(*v),
        Ok(_) => Err(ParseFitFileError::InvalidFieldValue(
            name.to_string(),
            "u16".to_string(),
        )),
        Err(e) => Err(e),
    }
}

fn get_u8(name: &str, entries: &[FitDataField]) -> errors::Result<u8> {
    match get_field(name, entries) {
        Ok(Value::UInt8(v)) => Ok(*v),
        Ok(_) => Err(ParseFitFileError::InvalidFieldValue(
            name.to_string(),
            "u8".to_string(),
        )),
        Err(e) => Err(e),
    }
}

fn get_timestamp(name: &str, entries: &[FitDataField]) -> errors::Result<DateTime<Local>> {
    match get_field(name, entries) {
        Ok(Value::Timestamp(v)) => Ok(*v),
        Ok(_) => Err(ParseFitFileError::InvalidFieldValue(
            name.to_string(),
            "timestamp".to_string(),
        )),
        Err(e) => Err(e),
    }
}

fn get_string(name: &str, entries: &[FitDataField]) -> errors::Result<String> {
    match get_field(name, entries) {
        Ok(Value::String(v)) => Ok(v.clone()),
        Ok(_) => Err(ParseFitFileError::InvalidFieldValue(
            name.to_string(),
            "string".to_string(),
        )),
        Err(e) => Err(e),
    }
}

fn get_i64(name: &str, entries: &[FitDataField]) -> errors::Result<i64> {
    match get_field(name, entries) {
        Ok(Value::SInt64(v)) => Ok(*v),
        Ok(_) => Err(ParseFitFileError::InvalidFieldValue(
            name.to_string(),
            "i64".to_string(),
        )),
        Err(e) => Err(e),
    }
}

fn get_field<'a>(name: &str, entries: &'a [FitDataField]) -> errors::Result<&'a Value> {
    match entries.iter().find(|e| e.name() == name).map(|e| e.value()) {
        Some(v) => Ok(v),
        None => Err(ParseFitFileError::MissingField(name.to_string())),
    }
}
