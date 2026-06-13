use crate::{
    garmin::models::{exercise::Exercise, serie::Serie, session::Session},
    models::{exercises::ExerciseListItem, records::RecordListItem, workouts::WorkoutListItem},
};

pub fn get_session_list() -> Vec<WorkoutListItem> {
    let sessions = Session::load_from_db().unwrap_or_default();

    sessions
        .into_iter()
        .map(|s| WorkoutListItem {
            name: s.workout.clone(),
            timestamp: s.timestamp.timestamp(),
            date: s.format_date().clone(),
        })
        .collect::<Vec<_>>()
}
//
//pub fn show_session_details(app: Weak<App>, timestamp: SharedString) {
//    let session = Session::find_by_id(&timestamp.to_string())
//        .unwrap()
//        .unwrap();
//
//    app.upgrade_in_event_loop(move |app| {
//        let details_arr: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Date").into());
//        items.push(SharedString::from(&session.format_date()).into());
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Total time").into());
//        items.push(SharedString::from(session.format_total_time()).into());
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Active time").into());
//        items.push(SharedString::from(session.format_active_time()).into());
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Active calories").into());
//        items.push(
//            SharedString::from(format!(
//                "{} Kcal",
//                session.total_calories - session.metabolic_calories
//            ))
//            .into(),
//        );
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Average heart rate").into());
//        items.push(SharedString::from(format!("{} BPM", session.avg_heart_rate)).into());
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Max heart rate").into());
//        items.push(SharedString::from(format!("{} BPM", session.max_heart_rate)).into());
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Volume").into());
//        items.push(SharedString::from(format!("{} Kg", session.get_volume())).into());
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("").into());
//        items.push(SharedString::from("").into());
//        details_arr.push(items.into());
//
//        for s in &session.series {
//            let items: Rc<VecModel<StandardListViewItem>> = Rc::new(VecModel::default());
//            items.push(SharedString::from(s.0.name.clone()).into());
//            items.push(
//                SharedString::from(
//                    s.1.iter()
//                        .map(|ser| format!("{}x{} Kg", ser.reps, ser.weight))
//                        .collect::<Vec<_>>()
//                        .join("\n"),
//                )
//                .into(),
//            );
//            details_arr.push(items.into());
//        }
//
//        app.set_session_details_name(session.workout.into());
//        app.set_session_detail(details_arr.into());
//        app.set_has_session_detail(true);
//    })
//    .unwrap();
//}
//
//pub fn import_fit_file(app: Weak<App>) {
//    if let Some(file) = FileDialog::new()
//        .add_filter("Garmin Fit", &["fit"])
//        .pick_file()
//    {
//        let mut err_txt = None;
//        println!("Selected {} file to import", file.display());
//        match Session::load_from_file(file) {
//            Ok(session) => match DATABASE_INST.lock() {
//                Ok(mut db) => {
//                    match db.run_in_transaction(|tx| {
//                        session.insert(tx)?;
//                        Ok(())
//                    }) {
//                        Ok(_) => {}
//                        Err(e) => {
//                            err_txt = Some(format!("Error writing to database: {}", e));
//                        }
//                    }
//                }
//                Err(e) => {
//                    err_txt = Some(format!("Error accesing to database: {}", e));
//                }
//            },
//            Err(e) => err_txt = Some(format!("Error parsing session: {}", e)),
//        }
//
//        app.upgrade_in_event_loop(|app| {
//            let error = SharedString::from(err_txt.unwrap_or("".to_string()));
//
//            app.set_error(error);
//            app.set_session_items(get_session_list());
//            app.set_exercise_items(get_exercise_list());
//            app.set_record_items(get_record_list().into());
//        })
//        .unwrap();
//    }
//}

pub fn get_exercise_list() -> Vec<ExerciseListItem> {
    let exercises = Exercise::load_from_db().unwrap_or_default();

    exercises
        .into_iter()
        .map(|s| ExerciseListItem {
            category: s.category.clone(),
            id: s.id,
            name: s.name.clone(),
        })
        .collect::<Vec<_>>()
}

pub fn get_record_list() -> Vec<RecordListItem> {
    let mut records = Vec::new();

    let exercises = Exercise::load_from_db().unwrap();
    for exercise in exercises {
        let pr = Serie::get_pr_for_exercise(&exercise).unwrap();
        records.push(RecordListItem {
            exercise: exercise.name,
            reps: pr.reps,
            weight: pr.weight,
            rm: pr.get_1rm_estimation(),
        });
    }

    records
}

//pub fn show_exercise_details(app: Weak<App>, category: SharedString, id: i32) {
//    let exercise = Exercise::load_by_cat_and_id(&category, id as u16)
//        .unwrap()
//        .unwrap();
//
//    let pr = Serie::get_pr_for_exercise(&exercise).unwrap();
//
//    app.upgrade_in_event_loop(move |app| {
//        let details_arr: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Personal record").into());
//        items.push(SharedString::from(format!("{}x{:.1} Kg", pr.reps, pr.weight)).into());
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("Estimated 1RM").into());
//        items.push(SharedString::from(format!("{:.1} Kg", pr.get_1rm_estimation())).into());
//        details_arr.push(items.into());
//
//        let items = Rc::new(VecModel::default());
//        items.push(SharedString::from("").into());
//        items.push(SharedString::from("").into());
//        details_arr.push(items.into());
//
//        let sessions = Session::load_from_db().unwrap();
//        for session in sessions {
//            if let Some(ser_str) = session
//                .series
//                .iter()
//                .filter_map(|s| {
//                    if s.0.category == exercise.category && s.0.id == exercise.id {
//                        Some(
//                            s.1.iter()
//                                .map(|ser| format!("{}x{} Kg", ser.reps, ser.weight))
//                                .collect::<Vec<String>>()
//                                .join("\n"),
//                        )
//                    } else {
//                        None
//                    }
//                })
//                .next()
//            {
//                let items = Rc::new(VecModel::default());
//                items.push(SharedString::from(session.format_date()).into());
//                items.push(SharedString::from(ser_str).into());
//                details_arr.push(items.into());
//            }
//        }
//
//        app.set_exercise_details_name(exercise.name.into());
//        app.set_exercise_detail(details_arr.into());
//        app.set_has_exercise_detail(true);
//    })
//    .unwrap();
//}
//
