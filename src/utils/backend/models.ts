//Auto generated file, do not edit manually

// From src-tauri/src/dto/app.rs:4
export enum AppEnvironment {
	Debug = "Debug",
	Release = "Release",
}

// From src-tauri/src/dto/body_metrics.rs:6
export interface BodyMetricListItem {
  date: number;
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

// From src-tauri/src/dao/settings.rs:132
export enum DistanceUnit {
	Kilometers = "Kilometers",
	Miles = "Miles",
}

// From src-tauri/src/dto/exercises.rs:33
export interface ExerciseDetails {
  category: string;
  id: number;
  name: string;
  pr_date: number;
  reps: number;
  rm: number;
  series: Record<string, SessionSerie[]>;
  weight: number;
  workouts: string[];
}

// From src-tauri/src/dto/exercises.rs:8
export interface ExerciseListItem {
  category: string;
  date: number;
  id: number;
  name: string;
  reps: number;
  rm: number;
  weight: number;
}

// From src-tauri/src/utils/translations.rs:10
export enum Languages {
	Spanish = "Spanish",
	English = "English",
}

// From src-tauri/src/dto/sessions.rs:63
export interface SessionDetails {
  active_time: number;
  coordinates: ([number, number] | null)[];
  device: string | null;
  heart_rates: (number | null)[];
  metabolic_calories: number;
  name: string;
  series: SessionSerie[];
  speeds: (number | null)[];
  sport: string;
  timestamp: number;
  total_calories: number;
  total_elapsed_time: number;
  training_load: number;
}

// From src-tauri/src/dto/sessions.rs:8
export interface SessionListItem {
  active_calories: number;
  name: string;
  sport: string;
  timestamp: number;
  training_load: number;
}

// From src-tauri/src/dto/sessions.rs:148
export interface SessionLocation {
  location: string;
  session: number;
}

// From src-tauri/src/dto/sessions.rs:40
export interface SessionSerie {
  ex_cat: string;
  ex_id: number;
  exercise: string;
  idx: number;
  reps: number;
  weight: number;
}

// From src-tauri/src/dto/sessions.rs:142
export interface SessionSeriesUpdate {
  series: SessionSerie[];
  timestamp: number;
}

// From src-tauri/src/dto/app.rs:15
export interface Settings {
  auto_sync: boolean;
  distance_unit: DistanceUnit;
  language: Languages;
  start_boot: boolean;
  weight_unit: WeightUnit;
}

// From src-tauri/src/dao/settings.rs:160
export enum WeightUnit {
	Kilograms = "Kilograms",
	Pounds = "Pounds",
}

// From src-tauri/src/dto/workouts.rs:33
export interface WorkoutDetails {
  avg_time: number;
  avg_volume: number;
  latest_session: number;
  name: string;
  session_count: number;
  sessions: WorkoutSession[];
}

// From src-tauri/src/dto/workouts.rs:6
export interface WorkoutListItem {
  avg_time: number;
  latest_session: number;
  name: string;
  sessions: number;
}

// From src-tauri/src/dto/workouts.rs:14
export interface WorkoutSession {
  date: number;
  time: number;
  vol_diff: string;
  volume: number;
}

