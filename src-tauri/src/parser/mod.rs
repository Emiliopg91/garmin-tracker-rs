use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use embedded_io_adapters::std::FromStd;
use rustyfit::{
    Decoder, DecoderEvent, StreamingIterator,
    profile::{
        mesgdef,
        typedef::{ExerciseCategory, MesgNum},
    },
    proto::Message,
};

use crate::{
    dao::{additional_data::AdditionalData, exercise::Exercise, serie::Serie, session::Session},
    utils::translations::{Languages, translate_and_replace},
};

use self::errors::ParseFitFileError;

pub mod errors;

/// Buckets the decoded FIT messages by message type in a single pass, converting each into
/// its typed `mesgdef` struct, so downstream extraction reads each bucket instead of
/// re-scanning and re-matching the whole message list once per field group.
#[derive(Default)]
struct GroupedEntries {
    session: Option<mesgdef::Session>,
    workout: Option<mesgdef::Workout>,
    exercise_titles: Vec<mesgdef::ExerciseTitle>,
    workout_steps: Vec<mesgdef::WorkoutStep>,
    sets: Vec<mesgdef::Set>,
    records: Vec<mesgdef::Record>,
}

impl GroupedEntries {
    /// Buckets FIT messages by message kind in a single pass. Branches are ordered by
    /// expected frequency (`RECORD` can number in the thousands per file, while
    /// `SESSION`/`WORKOUT` appear once) so the common case needs fewer comparisons.
    fn from_entries(entries: &[Message]) -> Self {
        let mut grouped = GroupedEntries::default();

        for entry in entries {
            match entry.num {
                MesgNum::RECORD => {
                    grouped.records.push(mesgdef::Record::from(entry));
                }
                MesgNum::SET => {
                    grouped.sets.push(mesgdef::Set::from(entry));
                }
                MesgNum::WORKOUT_STEP => {
                    grouped
                        .workout_steps
                        .push(mesgdef::WorkoutStep::from(entry));
                }
                MesgNum::EXERCISE_TITLE => {
                    grouped
                        .exercise_titles
                        .push(mesgdef::ExerciseTitle::from(entry));
                }
                MesgNum::SESSION => {
                    grouped.session = Some(mesgdef::Session::from(entry));
                }
                MesgNum::WORKOUT => {
                    grouped.workout = Some(mesgdef::Workout::from(entry));
                }
                _ => {}
            };
        }

        grouped
    }
}

/// Parses a `.FIT` activity file into a `Session` (with nested series, heart rate, GPS, and speed data). Falls back to reverse-geocoding the start GPS point for the workout name if the file has none.
pub(crate) fn load_from_file<P>(path: P, lang: Languages) -> errors::Result<Session>
where
    P: AsRef<Path>,
{
    let entries = stream_from_file(&path, true)?;

    #[cfg(debug_assertions)]
    debug_dump(&path, &entries);

    let grouped = GroupedEntries::from_entries(&entries);

    let session_entry = grouped
        .session
        .as_ref()
        .ok_or_else(|| ParseFitFileError::MissingField("session".to_string()))?;

    let timestamp = get_timestamp(session_entry)?;
    let sport = get_sport_profile_name(session_entry)?;
    let workout = get_workout_name(grouped.workout.as_ref());
    let total_elapsed_time = get_total_elapsed_time(session_entry)?;
    let active_time = get_active_time(session_entry);
    let training_load = get_training_load_peak(session_entry)?;
    let total_calories = get_total_calories(session_entry)?;
    let metabolic_calories = get_metabolic_calories(session_entry)?;
    let series = get_sets(&grouped, timestamp, lang).unwrap_or_default();
    let additional_data = get_additional_data(timestamp, &grouped.records)?;

    Ok(Session {
        workout,
        date: timestamp,
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

/// Decodes a `.FIT` file into its raw messages, streaming message-by-message from the decoder
/// instead of buffering the whole decoded file at once. When `filter_relevant` is `true`, only
/// the message kinds consumed by [`GroupedEntries`] are kept, discarding the rest as they are
/// streamed rather than after collecting the whole file into memory.
pub fn stream_from_file<P>(path: P, filter_relevant: bool) -> errors::Result<Vec<Message>>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();

    let file = File::open(path_ref)
        .map_err(|e| ParseFitFileError::FileOpening(path_ref.display().to_string(), e))?;
    let mut reader = FromStd::new(BufReader::new(file));

    let mut decoder = Decoder::new();
    let mut stream = decoder.stream(&mut reader);

    let mut messages = Vec::new();
    while let Some(event) = stream.next() {
        let event = event.map_err(|e| {
            ParseFitFileError::FileReading(path_ref.display().to_string(), Box::new(e))
        })?;

        if let DecoderEvent::Message(mesg) = event
            && (!filter_relevant
                || matches!(
                    mesg.num,
                    MesgNum::SESSION
                        | MesgNum::WORKOUT
                        | MesgNum::EXERCISE_TITLE
                        | MesgNum::WORKOUT_STEP
                        | MesgNum::SET
                        | MesgNum::RECORD
                ))
        {
            messages.push(mesg.clone());
        }
    }

    Ok(messages)
}

/// Debug-only helper: writes the raw parsed messages to `<file>.txt` for inspection.
#[cfg(debug_assertions)]
pub fn debug_dump<P>(path: P, entries: &[Message])
where
    P: AsRef<Path>,
{
    let dump_path = format!("{}.json", path.as_ref().display());
    match serde_json::to_string_pretty(entries) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&dump_path, json) {
                eprintln!("failed to write debug dump to {:?}: {e}", dump_path);
            }
        }
        Err(e) => {
            eprintln!("failed to serialize data: {e}");
        }
    }
}

/// Extracts the workout name from the `workout` FIT message, if present.
fn get_workout_name(wkt_entry: Option<&mesgdef::Workout>) -> String {
    match wkt_entry {
        None => String::new(),
        Some(entry) => {
            if entry.wkt_name.is_empty() {
                String::new()
            } else {
                entry.wkt_name.clone()
            }
        }
    }
}

/// Builds the list of strength-training sets (`Serie`s) for a session, resolving each set to its exercise via the workout steps.
fn get_sets(
    grouped: &GroupedEntries,
    timestamp: i64,
    lang: Languages,
) -> errors::Result<Vec<Serie>> {
    let exercises = get_exercises(&grouped.exercise_titles)?;
    let steps = get_steps(&grouped.workout_steps, &exercises, lang)?;

    let mut sets = Vec::new();

    let valid_sets = grouped.sets.iter().filter_map(|reg| {
        if reg.repetitions == u16::MAX || reg.weight == u16::MAX || reg.wkt_step_index.0 == u16::MAX
        {
            return None;
        }

        let reps = reg.repetitions;
        let weight = reg.weight_scaled().unwrap();
        let ex_idx = reg.wkt_step_index.0 as usize;
        let exercise = steps.get(ex_idx)?.as_ref()?;
        Some((exercise.clone(), reps, weight))
    });

    for (idx, (exercise, reps, weight)) in valid_sets.enumerate() {
        sets.push(Serie {
            session: timestamp,
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

/// Extracts additional data from the session's `record` messages, packing them into a `AdditionalData` entity.
fn get_additional_data(
    timestamp: i64,
    records: &[mesgdef::Record],
) -> errors::Result<Option<AdditionalData>> {
    let mut hrs = Vec::with_capacity(records.len());
    let mut cadences = Vec::with_capacity(records.len());
    let mut coords = Vec::with_capacity(records.len());
    let mut speeds = Vec::with_capacity(records.len());
    let mut powers = Vec::with_capacity(records.len());
    let mut respirations = Vec::with_capacity(records.len());

    records.iter().for_each(|entry| {
        hrs.push(
            (entry.heart_rate != AdditionalData::INVALID_HEAR_RATE).then_some(entry.heart_rate),
        );
        cadences.push((entry.cadence != AdditionalData::INVALID_CADENCE).then_some(entry.cadence));
        coords.push(
            if entry.position_lat != AdditionalData::INVALID_POSITION
                && entry.position_long != AdditionalData::INVALID_POSITION
            {
                Some((entry.position_lat, entry.position_long))
            } else {
                None
            },
        );
        powers.push((entry.power != AdditionalData::INVALID_POWER).then_some(entry.power));
        speeds.push(entry.enhanced_speed_scaled());
        respirations.push(entry.enhanced_respiration_rate_scaled());
    });

    let coords = if !coords.is_empty() && coords.iter().any(|e| e.is_some()) {
        let mut new_coords = Vec::new();
        let mut last_valid = (
            AdditionalData::INVALID_POSITION,
            AdditionalData::INVALID_POSITION,
        );

        for c in coords {
            if let Some(pos) = c {
                last_valid = pos;
            }
            new_coords.push(last_valid)
        }

        Some(new_coords)
    } else {
        None
    };

    let hrs = fill_invalid(&hrs, AdditionalData::INVALID_HEAR_RATE);
    let cadences = fill_invalid(&cadences, AdditionalData::INVALID_CADENCE);
    let powers = fill_invalid(&powers, AdditionalData::INVALID_POWER);
    let speeds = fill_invalid(&speeds, AdditionalData::INVALID_SPEED);
    let respirations = fill_invalid(&respirations, AdditionalData::INVALID_RESPIRATIONS);

    if hrs.is_some()
        || coords.is_some()
        || speeds.is_some()
        || cadences.is_some()
        || powers.is_some()
        || respirations.is_some()
    {
        Ok(Some(AdditionalData {
            session: timestamp,
            heart_rates: hrs,
            cadences,
            coordinates: coords.map(|coords| AdditionalData::build_coordinates_blob(&coords)),
            speeds: speeds.map(|speeds| AdditionalData::build_speeds_blob(&speeds)),
            powers: powers.map(|powers| AdditionalData::build_powers_blob(&powers)),
            respirations: respirations
                .map(|respirations| AdditionalData::build_respirations_blob(&respirations)),
        }))
    } else {
        Ok(None)
    }
}

fn fill_invalid<T: Copy>(vals: &[Option<T>], invalid: T) -> Option<Vec<T>> {
    if vals.iter().any(|v| v.is_some()) {
        Some(vals.iter().map(|v| v.unwrap_or(invalid)).collect())
    } else {
        None
    }
}

/// Resolves each workout step to its `Exercise`, by looking it up in the exercise titles parsed from the same file.
fn get_steps(
    workout_steps: &[mesgdef::WorkoutStep],
    exercises: &[Exercise],
    lang: Languages,
) -> errors::Result<Vec<Option<Exercise>>> {
    let lookup: HashMap<(u16, &str), &Exercise> = exercises
        .iter()
        .map(|e| ((e.id, e.category.as_str()), e))
        .collect();

    workout_steps
        .iter()
        .map(|reg| {
            if reg.exercise_category.0 == u16::MAX {
                return Ok(None);
            }

            let ex_cat = reg.exercise_category.to_string();
            let ex_id = get_exercise_name(reg.exercise_name);

            lookup
                .get(&(ex_id, ex_cat.as_str()))
                .map(|e| Some((*e).clone()))
                .ok_or_else(|| {
                    ParseFitFileError::GenericError(translate_and_replace(
                        "error_parser_unknown_exercise",
                        &[&ex_cat, &ex_id.to_string()],
                        lang,
                    ))
                })
        })
        .collect()
}

/// Parses the file's `exercise_title` messages into `Exercise` entities.
pub fn get_exercises(exercise_titles: &[mesgdef::ExerciseTitle]) -> errors::Result<Vec<Exercise>> {
    exercise_titles
        .iter()
        .map(|reg| {
            let category = get_exercise_category(reg.exercise_category)?;
            let name = reg
                .wkt_step_name
                .first()
                .cloned()
                .ok_or_else(|| ParseFitFileError::MissingField("wkt_step_name".to_string()))?;

            Ok(Exercise {
                id: get_exercise_name(reg.exercise_name),
                category,
                name,
            })
        })
        .collect()
}

/// `exercise_name` falls back to `1` when absent, matching the previous `fitparser`-based behavior.
fn get_exercise_name(v: u16) -> u16 {
    if v == u16::MAX { 1 } else { v }
}

fn get_exercise_category(v: ExerciseCategory) -> errors::Result<String> {
    if v.0 == u16::MAX {
        Err(ParseFitFileError::MissingField(
            "exercise_category".to_string(),
        ))
    } else {
        Ok(v.to_string())
    }
}

fn get_timestamp(session: &mesgdef::Session) -> errors::Result<i64> {
    session
        .timestamp
        .unix_timestamp()
        .ok_or_else(|| ParseFitFileError::MissingField("timestamp".to_string()))
}

fn get_sport_profile_name(session: &mesgdef::Session) -> errors::Result<String> {
    if session.sport_profile_name.is_empty() {
        Err(ParseFitFileError::MissingField(
            "sport_profile_name".to_string(),
        ))
    } else {
        Ok(session.sport_profile_name.clone())
    }
}

fn get_total_elapsed_time(session: &mesgdef::Session) -> errors::Result<f64> {
    session
        .total_elapsed_time_scaled()
        .ok_or_else(|| ParseFitFileError::MissingField("total_elapsed_time".to_string()))
}

fn get_active_time(session: &mesgdef::Session) -> f64 {
    session.active_time_scaled().unwrap_or(0_f64)
}

fn get_training_load_peak(session: &mesgdef::Session) -> errors::Result<f64> {
    session
        .training_load_peak_scaled()
        .ok_or_else(|| ParseFitFileError::MissingField("training_load_peak".to_string()))
}

fn get_total_calories(session: &mesgdef::Session) -> errors::Result<u16> {
    if session.total_calories == u16::MAX {
        Err(ParseFitFileError::MissingField(
            "total_calories".to_string(),
        ))
    } else {
        Ok(session.total_calories)
    }
}

fn get_metabolic_calories(session: &mesgdef::Session) -> errors::Result<u16> {
    if session.metabolic_calories == u16::MAX {
        Err(ParseFitFileError::MissingField(
            "metabolic_calories".to_string(),
        ))
    } else {
        Ok(session.metabolic_calories)
    }
}
