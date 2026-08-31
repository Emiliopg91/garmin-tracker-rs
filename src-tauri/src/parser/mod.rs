use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

use chrono::{DateTime, Local};
use fitparser::{FitDataField, FitDataRecord, Value, de::DecodeOption, profile};

use crate::{
    dao::{additional_data::AdditionalData, exercise::Exercise, serie::Serie, session::Session},
    utils::translations::translate_and_replace,
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
    /// Buckets FIT records by message kind in a single pass.
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

/// Parses a `.FIT` activity file into a `Session` (with nested series, heart rate, GPS, and speed data). Falls back to reverse-geocoding the start GPS point for the workout name if the file has none.
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
    let workout = get_workout_name(grouped.workout).unwrap_or_default();
    let total_elapsed_time = get_f64("total_elapsed_time", session_entry.fields())?;
    let active_time = get_f64("active_time", session_entry.fields()).unwrap_or(0.0);
    let training_load = get_f64("training_load_peak", session_entry.fields())?;
    let total_calories = get_u16("total_calories", session_entry.fields())?;
    let metabolic_calories = get_u16("metabolic_calories", session_entry.fields())?;
    let series = get_sets(&grouped, &timestamp).unwrap_or_default();
    let additional_data = get_additional_data(&timestamp, &grouped.records)?;

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
        additional_data,
    })
}

/// Reads and decodes a `.FIT` file into its raw records, dropping unknown fields/messages.
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

/// Debug-only helper: writes the raw parsed records to `<file>.txt` for inspection.
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

/// Extracts the workout name from the `workout` FIT record, if present.
fn get_workout_name(wkt_entry: Option<&FitDataRecord>) -> errors::Result<String> {
    let wkt_entry =
        wkt_entry.ok_or_else(|| ParseFitFileError::MissingField("workout".to_string()))?;

    get_string("wkt_name", wkt_entry.fields())
        .map_err(|_| ParseFitFileError::InvalidFieldValue("name".to_string(), "string".to_string()))
}

/// Builds the list of strength-training sets (`Serie`s) for a session, resolving each set to its exercise via the workout steps.
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

/// Extracts additional data from the session's `record` messages, packing them into a `HeartRate` entity.
fn get_additional_data(
    timestamp: &DateTime<Local>,
    records: &[&FitDataRecord],
) -> errors::Result<Option<AdditionalData>> {
    let mut hrs = Vec::new();
    let mut coords = Vec::new();
    let mut speeds = Vec::with_capacity(records.len());

    records.iter().for_each(|entry| {
        hrs.push(get_u8("heart_rate", entry.fields()).ok());
        coords.push(
            if let Ok(latitude) = get_i32("position_lat", entry.fields())
                && let Ok(longitude) = get_i32("position_long", entry.fields())
            {
                Some((latitude, longitude))
            } else {
                None
            },
        );
        speeds.push(get_f64("enhanced_speed", entry.fields()).ok());
    });

    let hrs = if !hrs.is_empty() && hrs.iter().find(|e| e.is_some()).is_some() {
        Some(
            hrs.iter()
                .map(|e| match e {
                    Some(v) => *v,
                    None => 0_u8,
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let coords = if !coords.is_empty() && coords.iter().find(|e| e.is_some()).is_some() {
        fn get_coord_for_idx(coords: &[Option<(i32, i32)>], idx: usize) -> (i32, i32) {
            let elem = coords[idx];
            match elem {
                Some((lat, long)) => (lat, long),
                None => {
                    if idx > 0 {
                        get_coord_for_idx(coords, idx - 1)
                    } else {
                        (
                            AdditionalData::INVALID_POSITION,
                            AdditionalData::INVALID_POSITION,
                        )
                    }
                }
            }
        }

        let mut new_coords = Vec::new();
        for i in 0..coords.len() {
            new_coords.push(get_coord_for_idx(&coords, i));
        }
        Some(new_coords)
    } else {
        None
    };

    let speeds = if !speeds.is_empty() && speeds.iter().find(|e| e.is_some()).is_some() {
        Some(
            speeds
                .iter()
                .map(|e| match e {
                    Some(v) => *v,
                    None => -1_f64,
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    if hrs.is_some() || coords.is_some() || speeds.is_some() {
        Ok(Some(AdditionalData {
            session: timestamp.timestamp(),
            heart_rates: hrs,
            coordinates: coords.map(|coords| AdditionalData::build_coordinates_blob(&coords)),
            speeds: speeds.map(|speeds| AdditionalData::build_speeds_blob(&speeds)),
        }))
    } else {
        Ok(None)
    }
}

/// Resolves each workout step to its `Exercise`, by looking it up in the exercise titles parsed from the same file.
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
                    ParseFitFileError::GenericError(translate_and_replace(
                        "error_parser_unknown_exercise",
                        &[&ex_cat, &ex_id.to_string()],
                    ))
                })
        })
        .collect()
}

/// Parses the file's `exercise_title` messages into `Exercise` entities.
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

/// Finds the raw field value named `name` among a record's fields.
fn get_field<'a>(name: &str, entries: &'a [FitDataField]) -> errors::Result<&'a Value> {
    entries
        .iter()
        .find(|e| e.name() == name)
        .map(|e| e.value())
        .ok_or_else(|| ParseFitFileError::MissingField(name.to_string()))
}
