/* eslint-disable */

//Auto generated file, do not edit manually

// Definition: src-tauri/src/garmin/models/devices.rs:5
export interface DeviceListItem {
	manufacturer: string;
	model: string;
	serial_number: string;
}

// Definition: src-tauri/src/garmin/models/exercises.rs:33
export interface ExerciseDetails {
	category: string;
	id: number;
	name: string;
	reps: number;
	weight: number;
	rm: number;
	workouts: string[];
	series: Record<string, SessionSerie[]>;
	pr_date: string;
}

// Definition: src-tauri/src/garmin/models/exercises.rs:8
export interface ExerciseListItem {
	category: string;
	id: number;
	name: string;
	reps: number;
	weight: number;
	rm: number;
	date: string;
}

// Definition: src-tauri/src/garmin/models/notifications.rs:4
export interface NotificationDefinition {
	title: string;
	body: string;
}

// Definition: src-tauri/src/garmin/models/sessions.rs:59
export interface SessionDetails {
	name: string;
	date: string;
	timestamp: string;
	total_elapsed_time: string;
	active_time: string;
	total_calories: number;
	metabolic_calories: number;
	avg_heart_rate: number;
	max_heart_rate: number;
	exercises: string[];
	series: Record<string, SessionSerie[]>;
}

// Definition: src-tauri/src/garmin/models/sessions.rs:8
export interface SessionListItem {
	name: string;
	date: string;
	timestamp: string;
	volume: number;
	exercises_num: number;
	series_num: number;
}

// Definition: src-tauri/src/garmin/models/sessions.rs:42
export interface SessionSerie {
	idx: number;
	reps: number;
	weight: number;
}

// Definition: src-tauri/src/garmin/models/sessions.rs:96
export interface SessionSeriesUpdate {
	timestamp: string;
	series: SessionSerie[];
}

// Definition: src-tauri/src/garmin/models/workouts.rs:33
export interface WorkoutDetails {
	name: string;
	latest_session: string;
	session_count: number;
	avg_time: string;
	avg_volume: number;
	sessions: WorkoutSession[];
}

// Definition: src-tauri/src/garmin/models/workouts.rs:6
export interface WorkoutListItem {
	name: string;
	latest_session: string;
	sessions: number;
	avg_time: string;
}

// Definition: src-tauri/src/garmin/models/workouts.rs:14
export interface WorkoutSession {
	date: string;
	volume: number;
	time: string;
	vol_diff: string;
}
