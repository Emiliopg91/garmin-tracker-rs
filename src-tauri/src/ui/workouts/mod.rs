pub mod models;

use std::collections::HashMap;

use tauri::AppHandle;

use crate::{
    garmin::database::dao::session::Session,
    ui::{
        notifications::{models::NotificationDefinition, show_notification},
        workouts::models::{WorkoutDetails, WorkoutListItem, WorkoutSession},
    },
};

#[tauri::command]
pub fn get_workout_list(app: AppHandle) -> Result<Vec<WorkoutListItem>, String> {
    let res: Result<Vec<WorkoutListItem>, String> = {
        let sessions = Session::load_from_db().map_err(|e| e.to_string())?;

        let mut count = HashMap::new();
        let mut latest = HashMap::new();
        let mut time: HashMap<_, _> = HashMap::new();
        sessions.iter().for_each(|s| {
            let entry = count.entry(s.workout.clone()).or_insert(0_u32);
            *entry += 1_u32;

            let entry = time.entry(s.workout.clone()).or_insert(0_f64);
            *entry += s.total_elapsed_time;

            let entry = latest.entry(s.workout.clone()).or_insert(s);
            *entry = if s.timestamp.timestamp() > entry.timestamp.timestamp() {
                s
            } else {
                entry
            }
        });

        let mut res = count
            .keys()
            .map(|k| WorkoutListItem {
                name: k.clone(),
                sessions: *count.get(k).unwrap(),
                avg_time: Session::format_duration(
                    (*time.get(k).unwrap() / (*count.get(k).unwrap() as f64)) as u64,
                ),
                latest_session: latest.get(k).unwrap().format_date(),
            })
            .collect::<Vec<_>>();

        res.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(res)
    };

    match res {
        Ok(l) => Ok(l),
        Err(e) => {
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error getting workouts list".to_string(),
                    body: e.to_string(),
                },
            );
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_workout_details(app: AppHandle, name: &str) -> Result<WorkoutDetails, String> {
    let res: Result<WorkoutDetails, String> = {
        let sessions = Session::find_by_workout(name).map_err(|e| e.to_string())?;

        let mut latest = sessions.first().unwrap();
        let mut count = 0_u32;
        let mut time = 0_f64;
        let mut volume = 0_f64;

        sessions.iter().for_each(|s| {
            if s.timestamp.timestamp() > latest.timestamp.timestamp() {
                latest = s;
            }
            count += 1;
            time += s.total_elapsed_time;
            volume += s.get_volume();
        });

        let mut details = WorkoutDetails {
            name: name.to_string(),
            avg_time: Session::format_duration((time / (sessions.len() as f64)) as u64),
            latest_session: latest.format_date(),
            avg_volume: volume / (sessions.len() as f64),
            session_count: count,
            sessions: sessions.iter().map(WorkoutSession::from).collect(),
        };

        for i in 0..details.sessions.len().saturating_sub(1) {
            let (left, right) = details.sessions.split_at_mut(i + 1);

            let current = &mut left[i];
            let previous = &right[0];

            current.vol_diff = format!(
                "{:+.2}%",
                (current.volume - previous.volume) / previous.volume * 100.0
            );
        }

        Ok(details)
    };

    match res {
        Ok(l) => Ok(l),
        Err(e) => {
            let _ = show_notification(
                app,
                NotificationDefinition {
                    title: "Error getting workout details".to_string(),
                    body: e.clone(),
                },
            );
            Err(e)
        }
    }
}
