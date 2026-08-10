//Auto generated file, do not edit manually

// From src-tauri/src/dto/app.rs:4
export enum AppEnvironment {
	Debug = "Debug",
	Release = "Release",
}

// From src-tauri/src/dto/body_metrics.rs:7
export interface BodyMetricListItem {
  date: string;
  fat_ratio: number;
  lean_mass: number;
  water_ratio: number;
  weight: number;
}

// From src-tauri/src/dto/devices.rs:7
export interface DeviceListItem {
  manufacturer: string;
  model: string;
  serial_number: string;
}

// From src-tauri/src/dto/exercises.rs:33
export interface ExerciseDetails {
  category: string;
  id: number;
  name: string;
  pr_date: string;
  reps: number;
  rm: number;
  series: Record<string, SessionSerie[]>;
  weight: number;
  workouts: string[];
}

// From src-tauri/src/dto/exercises.rs:8
export interface ExerciseListItem {
  category: string;
  date: string;
  id: number;
  name: string;
  reps: number;
  rm: number;
  weight: number;
}

// From src-tauri/src/dto/sessions.rs:69
export interface SessionDetails {
  active_time: string;
  avg_heart_rate: number;
  date: string;
  device: string | null;
  exercises: string[];
  heart_rates: number[];
  max_heart_rate: number;
  metabolic_calories: number;
  name: string;
  series: Record<string, SessionSerie[]>;
  sub_sport: string;
  timestamp: string;
  total_calories: number;
  total_elapsed_time: string;
  training_load: number;
  zones_times: string[];
}

// From src-tauri/src/dto/sessions.rs:12
export interface SessionListItem {
  active_calories: number;
  date: string;
  name: string;
  sub_sport: string;
  timestamp: string;
  training_load: number;
  volume: number;
}

// From src-tauri/src/dto/sessions.rs:48
export interface SessionSerie {
  ex_cat: string;
  ex_id: number;
  idx: number;
  reps: number;
  weight: number;
}

// From src-tauri/src/dto/sessions.rs:198
export interface SessionSeriesUpdate {
  series: SessionSerie[];
  timestamp: string;
}

// From src-tauri/src/dto/workouts.rs:33
export interface WorkoutDetails {
  avg_time: string;
  avg_volume: number;
  latest_session: string;
  name: string;
  session_count: number;
  sessions: WorkoutSession[];
}

// From src-tauri/src/dto/workouts.rs:6
export interface WorkoutListItem {
  avg_time: string;
  latest_session: string;
  name: string;
  sessions: number;
}

// From src-tauri/src/dto/workouts.rs:14
export interface WorkoutSession {
  date: string;
  time: string;
  vol_diff: string;
  volume: number;
}

