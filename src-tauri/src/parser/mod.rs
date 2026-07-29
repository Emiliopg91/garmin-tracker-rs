use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

use chrono::{DateTime, Local};
use fitparser::{FitDataField, FitDataRecord, Value, de::DecodeOption, profile};
use indexmap::IndexMap;

use crate::dao::{exercise::Exercise, heart_rate::HeartRate, serie::Serie, session::Session};

use self::errors::ParseFitFileError;

pub mod errors;

pub(crate) fn load_from_file<P>(path: P) -> errors::Result<Session>
where
    P: AsRef<Path>,
{
    let entries = read_from_file(&path)?
        .into_iter()
        .filter(|r| {
            matches!(
                r.kind(),
                profile::MesgNum::ExerciseTitle
                    | profile::MesgNum::Session
                    | profile::MesgNum::Workout
                    | profile::MesgNum::WorkoutStep
                    | profile::MesgNum::Set
                    | profile::MesgNum::Record
            )
        })
        .collect::<Vec<FitDataRecord>>();

    #[cfg(debug_assertions)]
    debug_dump(&path, &entries);

    let session_entry = entries
        .iter()
        .find(|r| r.kind() == profile::MesgNum::Session)
        .ok_or_else(|| ParseFitFileError::MissingField("session".to_string()))?;

    let timestamp = get_timestamp("timestamp", session_entry.fields())?;
    let sub_sport = get_string("sub_sport", session_entry.fields())?;
    let workout = if sub_sport == "strength_training" {
        get_workout_name(&entries)?
    } else {
        "".to_string()
    };
    let total_elapsed_time = get_f64("total_elapsed_time", session_entry.fields())?;
    let active_time = get_f64("active_time", session_entry.fields()).unwrap_or(0.0);
    let training_load = get_f64("training_load_peak", session_entry.fields())?;
    let total_calories = get_u16("total_calories", session_entry.fields())?;
    let metabolic_calories = get_u16("metabolic_calories", session_entry.fields())?;
    let avg_heart_rate = get_u8("avg_heart_rate", session_entry.fields())?;
    let max_heart_rate = get_u8("max_heart_rate", session_entry.fields())?;
    let series = if sub_sport == "strength_training" {
        get_sets(&entries, &timestamp)?
    } else {
        IndexMap::new()
    };
    let heart_rates = get_heart_rate(&entries, &timestamp)?;

    Ok(Session {
        workout,
        date: timestamp.timestamp(),
        total_elapsed_time,
        active_time,
        total_calories,
        metabolic_calories,
        avg_heart_rate,
        max_heart_rate,
        series,
        training_load,
        sub_sport,
        heart_rates,
    })
}

pub fn read_from_file<P>(path: P) -> errors::Result<Vec<FitDataRecord>>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();

    let mut fp = File::open(path_ref)
        .map_err(|e| ParseFitFileError::FileOpening(path_ref.display().to_string(), e))?;

    let mut options = HashSet::<DecodeOption>::new();
    options.insert(DecodeOption::DropUnknownFields);
    options.insert(DecodeOption::DropUnknownMessages);
    fitparser::de::from_reader_with_options(&mut fp, &options)
        .map_err(|e| ParseFitFileError::FileReading(path_ref.display().to_string(), e))
}

#[cfg(debug_assertions)]
pub fn debug_dump<P>(path: P, entries: &[FitDataRecord])
where
    P: AsRef<Path>,
{
    let dump_path = format!("{}.txt", path.as_ref().display());
    if let Err(e) = std::fs::write(&dump_path, format!("{:#?}", entries)) {
        eprintln!("failed to write debug dump to {:?}: {e}", dump_path);
    }
}

fn get_workout_name(entries: &[FitDataRecord]) -> errors::Result<String> {
    let wkt_entry = entries
        .iter()
        .find(|r| r.kind() == profile::MesgNum::Workout)
        .ok_or_else(|| ParseFitFileError::MissingField("workout".to_string()))?;

    get_string("wkt_name", wkt_entry.fields())
        .map_err(|_| ParseFitFileError::InvalidFieldValue("name".to_string(), "string".to_string()))
}

fn get_sets(
    entries: &[FitDataRecord],
    timestamp: &DateTime<Local>,
) -> errors::Result<IndexMap<Exercise, Vec<Serie>>> {
    let exercises = get_exercises(entries)?;
    let steps = get_steps(entries, &exercises)?;

    let mut sets = IndexMap::<Exercise, Vec<Serie>>::new();

    let valid_sets = entries
        .iter()
        .filter(|r| r.kind() == profile::MesgNum::Set)
        .filter_map(|reg| {
            let reps = get_u16("repetitions", reg.fields()).ok()?;
            let weight = get_f64("weight", reg.fields()).ok()?;
            let ex_idx = get_i64("wkt_step_index", reg.fields()).ok()?;
            let exercise = steps.get(ex_idx as usize)?.as_ref()?;
            Some((exercise.clone(), reps, weight))
        });

    for (idx, (exercise, reps, weight)) in valid_sets.enumerate() {
        sets.entry(exercise.clone()).or_default().push(Serie {
            session: timestamp.timestamp(),
            idx: idx as u8,
            ex_cat: exercise.category.clone(),
            ex_id: exercise.id,
            reps,
            weight,
            pr: false,
        });
    }

    Ok(sets)
}

fn get_heart_rate(
    entries: &[FitDataRecord],
    timestamp: &DateTime<Local>,
) -> errors::Result<Vec<HeartRate>> {
    let mut hrs = Vec::new();

    let entries = entries
        .iter()
        .filter(|e| matches!(e.kind(), profile::MesgNum::Record))
        .cloned()
        .collect::<Vec<FitDataRecord>>();

    entries.iter().for_each(|entry| {
        if let Ok(val) = get_u8("heart_rate", entry.fields()) {
            hrs.push(HeartRate {
                session: timestamp.timestamp(),
                idx: hrs.len() as u32,
                hr: val,
            });
        }
    });

    Ok(hrs)
}

fn get_steps(
    entries: &[FitDataRecord],
    exercises: &[Exercise],
) -> errors::Result<Vec<Option<Exercise>>> {
    let lookup: HashMap<(u16, &str), &Exercise> = exercises
        .iter()
        .map(|e| ((e.id, e.category.as_str()), e))
        .collect();

    entries
        .iter()
        .filter(|r| r.kind() == profile::MesgNum::WorkoutStep)
        .map(|reg| {
            let Ok(ex_cat) = get_string("exercise_category", reg.fields()) else {
                return Ok(None);
            };

            let ex_id = get_u16("exercise_name", reg.fields()).unwrap_or(1);

            lookup
                .get(&(ex_id, ex_cat.as_str()))
                .map(|e| Some((*e).clone()))
                .ok_or_else(|| {
                    ParseFitFileError::GenericError(format!(
                        "Unknown exercise with category {} and id {}",
                        ex_cat, ex_id
                    ))
                })
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

macro_rules! typed_getter {
    ($fn_name:ident, $variant:ident, $ret:ty, $label:literal) => {
        fn $fn_name(name: &str, entries: &[FitDataField]) -> errors::Result<$ret> {
            match get_field(name, entries)? {
                Value::$variant(v) => Ok(v.clone()),
                _ => Err(ParseFitFileError::InvalidFieldValue(
                    name.to_string(),
                    $label.to_string(),
                )),
            }
        }
    };
}

typed_getter!(get_f64, Float64, f64, "f64");
typed_getter!(get_u16, UInt16, u16, "u16");
typed_getter!(get_u8, UInt8, u8, "u8");
typed_getter!(get_timestamp, Timestamp, DateTime<Local>, "timestamp");
typed_getter!(get_string, String, String, "string");
typed_getter!(get_i64, SInt64, i64, "i64");

fn get_field<'a>(name: &str, entries: &'a [FitDataField]) -> errors::Result<&'a Value> {
    entries
        .iter()
        .find(|e| e.name() == name)
        .map(|e| e.value())
        .ok_or_else(|| ParseFitFileError::MissingField(name.to_string()))
}
