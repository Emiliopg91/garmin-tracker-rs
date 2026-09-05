use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use embedded_io_adapters::std::FromStd;
use rustyfit::{
    Decoder, DecoderEvent, StreamDecoder, StreamingIterator,
    profile::{mesgdef, typedef::MesgNum},
    proto::Message,
};

use tauri_plugin_log::log::warn;

use crate::dao::{
    additional_data::AdditionalData, exercise::Exercise, exercise_category, serie::Serie,
    session::Session, sport::Sport, sub_sport::SubSport, workout::Workout,
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

pub struct FitParser<'a> {
    path: &'a Path,
    stream: StreamDecoder<'a, FromStd<BufReader<File>>>,
}

impl<'a> FitParser<'a> {
    pub fn from_file<P>(path: &'a P, decoder: &'a mut Decoder) -> errors::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path_ref = path.as_ref();

        let file = File::open(path_ref)
            .map_err(|e| ParseFitFileError::FileOpening(path_ref.display().to_string(), e))?;
        let reader = FromStd::new(BufReader::new(file));

        Ok(Self {
            path: path_ref,
            stream: decoder.stream(reader),
        })
    }

    /// Parses a `.FIT` activity file into a `Session` (with nested series, heart rate, GPS, and speed data). Falls back to reverse-geocoding the start GPS point for the workout name if the file has none.
    pub(crate) fn parse_session(mut self) -> errors::Result<Session> {
        let mut entries = Vec::new();
        while let Some(event) = self.stream.next() {
            let event = event.map_err(|e| {
                ParseFitFileError::FileReading(self.path.display().to_string(), Box::new(e))
            })?;

            if let DecoderEvent::Message(mesg) = event
                && matches!(
                    mesg.num,
                    MesgNum::SESSION
                        | MesgNum::WORKOUT
                        | MesgNum::EXERCISE_TITLE
                        | MesgNum::WORKOUT_STEP
                        | MesgNum::SET
                        | MesgNum::RECORD
                )
            {
                entries.push(mesg.clone());
            }
        }

        let grouped = GroupedEntries::from_entries(&entries);

        let session_entry = grouped
            .session
            .as_ref()
            .ok_or_else(|| ParseFitFileError::MissingField("session".to_string()))?;

        let timestamp = Self::get_timestamp(session_entry)?;
        let sub_sport_obj = Self::get_sub_sport(session_entry)?;
        let workout = grouped.workout.clone().map(|w| w.wkt_name);
        let total_elapsed_time = Self::get_total_elapsed_time(session_entry)?;
        let active_time = Self::get_active_time(session_entry);
        let training_load = Self::get_training_load_peak(session_entry)?;
        let total_calories = Self::get_total_calories(session_entry)?;
        let metabolic_calories = Self::get_metabolic_calories(session_entry)?;
        let series = Self::get_sets(&grouped, timestamp).unwrap_or_default();
        let additional_data = Self::get_additional_data(timestamp, &grouped.records)?;

        Ok(Session {
            date: timestamp,
            name: workout.clone().unwrap_or_default(),
            workout_obj: workout.as_ref().map(|o| Workout {
                name: o.to_string(),
            }),
            workout,
            total_elapsed_time,
            active_time,
            total_calories,
            metabolic_calories,
            series,
            training_load,
            sport: sub_sport_obj.sport,
            sub_sport: sub_sport_obj.id,
            sub_sport_obj: Some(sub_sport_obj),
            device: None,
            device_obj: None,
            additional_data,
        })
    }

    /// Debug-only helper: writes the raw parsed messages to `<file>.json` for inspection.
    #[cfg(debug_assertions)]
    pub fn debug_dump(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let dump_path = format!("{}.json", self.path.display());
        let mut entries = Vec::new();

        while let Some(event) = self.stream.next() {
            let event = event.map_err(|e| {
                ParseFitFileError::FileReading(self.path.display().to_string(), Box::new(e))
            })?;

            if let DecoderEvent::Message(mesg) = event
                && matches!(
                    mesg.num,
                    MesgNum::SESSION
                        | MesgNum::WORKOUT
                        | MesgNum::EXERCISE_TITLE
                        | MesgNum::WORKOUT_STEP
                        | MesgNum::SET
                        | MesgNum::RECORD
                )
            {
                entries.push(mesg.clone());
            }
        }

        let json = serde_json::to_string_pretty(&entries)?;
        std::fs::write(&dump_path, json)?;

        Ok(())
    }

    /// Builds the list of strength-training sets (`Serie`s) for a session, resolving each set to its exercise via the workout steps.
    fn get_sets(grouped: &GroupedEntries, timestamp: i64) -> errors::Result<Vec<Serie>> {
        let exercises = Self::get_exercises(&grouped.exercise_titles)?;
        let steps = Self::get_steps(&grouped.workout_steps, &exercises);

        let mut sets = Vec::new();

        let valid_sets = grouped.sets.iter().filter_map(|reg| {
            if reg.repetitions == u16::MAX || reg.wkt_step_index.0 == u16::MAX {
                return None;
            }

            let reps = reg.repetitions;
            let weight = reg.weight_scaled()?;
            let ex_idx = reg.wkt_step_index.0 as usize;
            let exercise = steps.get(ex_idx)?.as_ref()?;
            Some((exercise.clone(), reps, weight))
        });

        for (idx, (exercise, reps, weight)) in valid_sets.enumerate() {
            sets.push(Serie {
                session: timestamp,
                idx: idx as u8,
                ex_cat: exercise.category,
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
            cadences
                .push((entry.cadence != AdditionalData::INVALID_CADENCE).then_some(entry.cadence));
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

        let hrs = Self::fill_invalid(&hrs, AdditionalData::INVALID_HEAR_RATE);
        let cadences = Self::fill_invalid(&cadences, AdditionalData::INVALID_CADENCE);
        let powers = Self::fill_invalid(&powers, AdditionalData::INVALID_POWER);
        let speeds = Self::fill_invalid(&speeds, AdditionalData::INVALID_SPEED);
        let respirations = Self::fill_invalid(&respirations, AdditionalData::INVALID_RESPIRATIONS);

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
    /// A step whose exercise can't be resolved is skipped (logged) rather than failing the whole session's series,
    /// so one unresolved step doesn't wipe out every other set in the session.
    fn get_steps(
        workout_steps: &[mesgdef::WorkoutStep],
        exercises: &[Exercise],
    ) -> Vec<Option<Exercise>> {
        let lookup: HashMap<(u16, u16), &Exercise> =
            exercises.iter().map(|e| ((e.id, e.category), e)).collect();

        workout_steps
            .iter()
            .map(|reg| {
                if reg.exercise_category.0 == u16::MAX {
                    return None;
                }

                let ex_cat = reg.exercise_category.0;
                let ex_id = Self::get_exercise_name(reg.exercise_name);

                let exercise = lookup.get(&(ex_id, ex_cat)).map(|e| (*e).clone());
                if exercise.is_none() {
                    warn!("{}", ParseFitFileError::UnknownExercise(ex_cat, ex_id));
                }
                exercise
            })
            .collect()
    }

    /// Parses the file's `exercise_title` messages into `Exercise` entities.
    pub fn get_exercises(
        exercise_titles: &[mesgdef::ExerciseTitle],
    ) -> errors::Result<Vec<Exercise>> {
        exercise_titles
            .iter()
            .map(|reg| {
                let category = if reg.exercise_category.0 == u16::MAX {
                    Err(ParseFitFileError::MissingField(
                        "exercise_category".to_string(),
                    ))
                } else {
                    Ok(reg.exercise_category.0)
                }?;
                let name = reg.exercise_name;

                Ok(Exercise {
                    category,
                    id: name,
                    exercise_category: Some(exercise_category::ExerciseCategory { id: category }),
                })
            })
            .collect()
    }

    /// `exercise_name` falls back to `1` when absent, matching the previous `fitparser`-based behavior.
    fn get_exercise_name(v: u16) -> u16 {
        if v == u16::MAX { 1 } else { v }
    }

    fn get_timestamp(session: &mesgdef::Session) -> errors::Result<i64> {
        session
            .timestamp
            .unix_timestamp()
            .ok_or_else(|| ParseFitFileError::MissingField("timestamp".to_string()))
    }

    fn get_sub_sport(session: &mesgdef::Session) -> errors::Result<SubSport> {
        let sport_val = session.sport;
        let sub_sport_val = session.sub_sport;
        if sport_val.0 != u8::MAX {
            if sub_sport_val.0 != u8::MAX {
                Ok(SubSport {
                    id: sub_sport_val.0,
                    sport: sport_val.0,
                    sport_obj: Some(Sport { id: sport_val.0 }),
                })
            } else {
                Err(ParseFitFileError::MissingField("sub_sport".to_string()))
            }
        } else {
            Err(ParseFitFileError::MissingField("sport".to_string()))
        }
    }

    fn get_total_elapsed_time(session: &mesgdef::Session) -> errors::Result<u32> {
        session
            .total_elapsed_time_scaled()
            .map(|v| v.round() as u32)
            .ok_or_else(|| ParseFitFileError::MissingField("total_elapsed_time".to_string()))
    }

    fn get_active_time(session: &mesgdef::Session) -> u32 {
        session
            .active_time_scaled()
            .map(|v| v.round() as u32)
            .unwrap_or(0_u32)
    }

    fn get_training_load_peak(session: &mesgdef::Session) -> errors::Result<u16> {
        session
            .training_load_peak_scaled()
            .map(|e| e.round() as u16)
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
}
