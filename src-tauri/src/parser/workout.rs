use crate::{
    dao::{workout::Workout, workout_step::WorkoutStep},
    parser::{
        FitParser,
        errors::{self, ParseFitFileError},
    },
};
use rustyfit::{
    DecoderEvent, StreamingIterator,
    profile::{
        mesgdef,
        typedef::{Intensity, MesgNum, WktStepDuration},
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
                            name = Some(workout_obj.wkt_name).unwrap();
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
                                    Intensity::REST => match step_obj.duration_type {
                                        WktStepDuration::TIME => {
                                            steps.push(WorkoutStep::rest(
                                                &name,
                                                idx,
                                                step_obj.duration_value,
                                            ));
                                        }
                                        _ => {}
                                    },
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
                                            WktStepDuration::OPEN | _ => None,
                                        };
                                        steps.push(WorkoutStep::exercise(
                                            &name, idx, ex_cat, ex_id, weight, reps,
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

        Ok(Workout {
            name: name,
            enabled: true,
            steps,
        })
    }
}
