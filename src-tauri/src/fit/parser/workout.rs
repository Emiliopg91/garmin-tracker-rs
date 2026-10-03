use crate::{
    dao::{workout::Workout, workout_step::WorkoutStep},
    fit::parser::{
        FitParser,
        errors::{self, ParseFitFileError},
    },
};
use rustyfit::{
    DecoderEvent, StreamingIterator,
    profile::{
        mesgdef,
        typedef::{Intensity, MesgNum, Sport, SubSport, WktStepDuration},
    },
};

impl TryFrom<FitParser> for Workout {
    type Error = errors::ParseFitFileError;
    fn try_from(mut value: FitParser) -> Result<Self, Self::Error> {
        let path_string = value.borrow_path().display().to_string();
        let mut name = "".to_string();
        let mut steps = Vec::new();

        value.with_stream_mut(|stream| -> errors::Result<()> {
            while let Some(event) = stream.next() {
                let event = event.map_err(|e| {
                    ParseFitFileError::FileReading(path_string.clone(), Box::new(e))
                })?;
                if let DecoderEvent::Message(msg) = event {
                    match msg.num {
                        MesgNum::WORKOUT => {
                            let workout_obj = mesgdef::Workout::from(msg);
                            if workout_obj.sport != Sport::TRAINING
                                || workout_obj.sub_sport != SubSport::STRENGTH_TRAINING
                            {
                                return Err(ParseFitFileError::NotAWorkout());
                            }

                            name = workout_obj.wkt_name;
                        }
                        MesgNum::WORKOUT_STEP => {
                            let step_obj = mesgdef::WorkoutStep::from(msg);
                            let idx = step_obj.message_index.0;
                            match step_obj.duration_type {
                                WktStepDuration::REPEAT_UNTIL_STEPS_CMPLT => {
                                    steps.push(WorkoutStep::repeat(
                                        &name,
                                        idx,
                                        step_obj.duration_value as u16,
                                        step_obj.target_value as u16,
                                    ));
                                }
                                _ => match step_obj.intensity {
                                    Intensity::REST => {
                                        let duration = match step_obj.duration_type {
                                            WktStepDuration::TIME => Some(step_obj.duration_value),
                                            _ => None,
                                        };
                                        steps.push(WorkoutStep::rest(&name, idx, duration));
                                    }
                                    Intensity::ACTIVE => {
                                        let ex_cat = step_obj.exercise_category.0;
                                        let ex_id = if step_obj.exercise_name == u16::MAX {
                                            1
                                        } else {
                                            step_obj.exercise_name
                                        };
                                        let weight =
                                            step_obj.exercise_weight_scaled().unwrap_or(0_f64)
                                                as f32;
                                        let reps = match step_obj.duration_type {
                                            WktStepDuration::REPS => {
                                                Some(step_obj.duration_value as u16)
                                            }
                                            _ => None,
                                        };
                                        let time = match step_obj.duration_type {
                                            WktStepDuration::TIME => Some(step_obj.duration_value),
                                            _ => None,
                                        };
                                        steps.push(WorkoutStep::exercise(
                                            &name, idx, ex_cat, ex_id, weight, reps, time,
                                        ));
                                    }
                                    _ => {}
                                },
                            }
                        }
                        _ => (),
                    }
                }
            }

            Ok(())
        })?;

        steps.sort_by_key(|s| s.idx);
        let old_idxs: Vec<u16> = steps.iter().map(|s| s.idx).collect();

        for (new_idx, step) in steps.iter_mut().enumerate() {
            step.idx = new_idx as u16;
            if let Some(begin) = step.begin_idx {
                step.begin_idx = Some(old_idxs.partition_point(|&old| old < begin) as u16);
            }
        }

        if !name.is_empty() {
            Ok(Workout {
                name,
                enabled: true,
                steps,
            })
        } else {
            Err(ParseFitFileError::MissingField("name".to_string()))
        }
    }
}
