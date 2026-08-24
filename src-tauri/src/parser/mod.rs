use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

use chrono::{DateTime, Local};
use fitparser::{FitDataField, FitDataRecord, Value, de::DecodeOption, profile};
use garmin_tracker_rs_macros::translate;

use crate::{
    dao::{
        exercise::Exercise, gps_coordinates::GpsCoordinates, heart_rate::HeartRate, serie::Serie,
        session::Session,
    },
    logic::sessions::get_location_from_coordinates,
    utils::constants,
};

use self::errors::ParseFitFileError;

pub mod errors;

/// Buckets the (already kind-filtered) entries by message type in a single
/// pass, so downstream extraction reads each bucket instead of re-scanning
/// and re-filtering the whole entry list once per field group.
#[derive(Default)]
struct GroupedEntries<'a> {
    session: Option<&'a FitDataRecord>,
    workout: Option<&'a FitDataRecord>,
    exercise_titles: Vec<&'a FitDataRecord>,
    workout_steps: Vec<&'a FitDataRecord>,
    sets: Vec<&'a FitDataRecord>,
    records: Vec<&'a FitDataRecord>,
}

impl<'a> GroupedEntries<'a> {
    fn from_entries(entries: &'a [FitDataRecord]) -> Self {
        let mut grouped = GroupedEntries::default();

        for entry in entries {
            match entry.kind() {
                profile::MesgNum::Session => grouped.session = Some(entry),
                profile::MesgNum::Workout => grouped.workout = Some(entry),
                profile::MesgNum::ExerciseTitle => grouped.exercise_titles.push(entry),
                profile::MesgNum::WorkoutStep => grouped.workout_steps.push(entry),
                profile::MesgNum::Set => grouped.sets.push(entry),
                profile::MesgNum::Record => grouped.records.push(entry),
                _ => {}
            }
        }

        grouped
    }
}

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

    let grouped = GroupedEntries::from_entries(&entries);

    let session_entry = grouped
        .session
        .ok_or_else(|| ParseFitFileError::MissingField("session".to_string()))?;

    let timestamp = get_timestamp("timestamp", session_entry.fields())?;
    let sport = get_string("sport_profile_name", session_entry.fields())?;
    let mut workout = get_workout_name(grouped.workout).unwrap_or_default();
    let total_elapsed_time = get_f64("total_elapsed_time", session_entry.fields())?;
    let active_time = get_f64("active_time", session_entry.fields()).unwrap_or(0.0);
    let training_load = get_f64("training_load_peak", session_entry.fields())?;
    let total_calories = get_u16("total_calories", session_entry.fields())?;
    let metabolic_calories = get_u16("metabolic_calories", session_entry.fields())?;
    let series = get_sets(&grouped, &timestamp).unwrap_or_default();
    let heart_rates = get_heart_rate(&timestamp, &grouped.records)?;
    let gps_coordinates = get_gps_coordinates(&timestamp, &grouped.records)?;
    if workout.is_empty()
        && let Some(coords) = gps_coordinates.clone()
    {
        let coords = coords.normalize();
        if let Some(start_point) = coords.first() {
            workout = get_location_from_coordinates(
                start_point.0 as f64 * constants::SEMICIRCLE_TO_DEGREES,
                start_point.1 as f64 * constants::SEMICIRCLE_TO_DEGREES,
            )
        }
    }

    Ok(Session {
        workout,
        date: timestamp.timestamp(),
        total_elapsed_time,
        active_time,
        total_calories,
        metabolic_calories,
        series,
        training_load,
        sport,
        device: None,
        device_obj: None,
        heart_rates,
        gps_coordinates,
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

fn get_workout_name(wkt_entry: Option<&FitDataRecord>) -> errors::Result<String> {
    let wkt_entry =
        wkt_entry.ok_or_else(|| ParseFitFileError::MissingField("workout".to_string()))?;

    get_string("wkt_name", wkt_entry.fields())
        .map_err(|_| ParseFitFileError::InvalidFieldValue("name".to_string(), "string".to_string()))
}

fn get_sets(grouped: &GroupedEntries, timestamp: &DateTime<Local>) -> errors::Result<Vec<Serie>> {
    let exercises = get_exercises(&grouped.exercise_titles)?;
    let steps = get_steps(&grouped.workout_steps, &exercises)?;

    let mut sets = Vec::new();

    let valid_sets = grouped.sets.iter().filter_map(|reg| {
        let reps = get_u16("repetitions", reg.fields()).ok()?;
        let weight = get_f64("weight", reg.fields()).ok()?;
        let ex_idx = get_i64("wkt_step_index", reg.fields()).ok()?;
        let exercise = steps.get(ex_idx as usize)?.as_ref()?;
        Some((exercise.clone(), reps, weight))
    });

    for (idx, (exercise, reps, weight)) in valid_sets.enumerate() {
        sets.push(Serie {
            session: timestamp.timestamp(),
            idx: idx as u8,
            ex_cat: exercise.category.clone(),
            ex_id: exercise.id,
            reps,
            weight,
            pr: false,
            exercise: Some(exercise),
        });
    }

    Ok(sets)
}

fn get_heart_rate(
    timestamp: &DateTime<Local>,
    records: &[&FitDataRecord],
) -> errors::Result<Option<HeartRate>> {
    let mut hrs = Vec::with_capacity(records.len());

    records.iter().for_each(|entry| {
        if let Ok(val) = get_u8("heart_rate", entry.fields()) {
            hrs.push(val);
        }
    });

    Ok(if hrs.is_empty() {
        None
    } else {
        Some(HeartRate {
            session: timestamp.timestamp(),
            records: hrs,
        })
    })
}

fn get_gps_coordinates(
    timestamp: &DateTime<Local>,
    records: &[&FitDataRecord],
) -> errors::Result<Option<GpsCoordinates>> {
    let mut coords = Vec::with_capacity(records.len());
    let mut start_point = None;

    records.iter().for_each(|entry| {
        if let Ok(latitude) = get_i32("position_lat", entry.fields())
            && let Ok(longitude) = get_i32("position_long", entry.fields())
        {
            coords.extend_from_slice(&latitude.to_be_bytes());
            coords.extend_from_slice(&longitude.to_be_bytes());

            if start_point.is_none() {
                start_point = Some((latitude, longitude));
            }
        }
    });

    Ok(if coords.is_empty() {
        None
    } else {
        Some(GpsCoordinates {
            session: timestamp.timestamp(),
            records: coords,
        })
    })
}

fn get_steps(
    workout_steps: &[&FitDataRecord],
    exercises: &[Exercise],
) -> errors::Result<Vec<Option<Exercise>>> {
    let lookup: HashMap<(u16, &str), &Exercise> = exercises
        .iter()
        .map(|e| ((e.id, e.category.as_str()), e))
        .collect();

    workout_steps
        .iter()
        .map(|reg| {
            let Ok(ex_cat) = get_string("exercise_category", reg.fields()) else {
                return Ok(None);
            };

            let ex_id = get_u16("exercise_name", reg.fields()).unwrap_or(1);

            lookup
                .get(&(ex_id, ex_cat.as_str()))
                .map(|e| Some((*e).clone()))
                .ok_or_else(|| {
                    ParseFitFileError::GenericError(translate!(
                        "error_parser_unknown_exercise",
                        ex_cat,
                        ex_id
                    ))
                })
        })
        .collect()
}

pub fn get_exercises(exercise_titles: &[&FitDataRecord]) -> errors::Result<Vec<Exercise>> {
    exercise_titles
        .iter()
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
typed_getter!(get_i32, SInt32, i32, "i32");

fn get_field<'a>(name: &str, entries: &'a [FitDataField]) -> errors::Result<&'a Value> {
    entries
        .iter()
        .find(|e| e.name() == name)
        .map(|e| e.value())
        .ok_or_else(|| ParseFitFileError::MissingField(name.to_string()))
}
