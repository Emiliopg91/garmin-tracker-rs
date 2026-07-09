//Auto generated file, do not edit manually

// From src-tauri/src/ui/app/models.rs:4
export enum AppEnvironment {
  Debug = "Debug",
  Release = "Release",
}

// From src-tauri/src/ui/devices/models.rs:7
export interface DeviceListItem {
  manufacturer: string;
  model: string;
  serial_number: string;
}

// From src-tauri/src/ui/exercises/models.rs:33
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

// From src-tauri/src/ui/exercises/models.rs:8
export interface ExerciseListItem {
  category: string;
  date: string;
  id: number;
  name: string;
  reps: number;
  rm: number;
  weight: number;
}

// From src-tauri/src/ui/app/models.rs:10
export enum LogLevel {
  Debug = "Debug",
  Info = "Info",
  Warn = "Warn",
  Error = "Error",
}

// From src-tauri/src/ui/notifications/models.rs:4
export enum NotificationKind {
  Temporal = "Temporal",
  Persistant = "Persistant",
}

// From src-tauri/src/ui/sessions/models.rs:63
export interface SessionDetails {
  active_time: string;
  avg_heart_rate: number;
  date: string;
  exercises: string[];
  max_heart_rate: number;
  metabolic_calories: number;
  name: string;
  series: Record<string, SessionSerie[]>;
  timestamp: string;
  total_calories: number;
  total_elapsed_time: string;
}

// From src-tauri/src/ui/sessions/models.rs:8
export interface SessionListItem {
  avg_heart_rate: number;
  date: string;
  exercises_num: number;
  max_heart_rate: number;
  name: string;
  series_num: number;
  timestamp: string;
  volume: number;
}

// From src-tauri/src/ui/sessions/models.rs:46
export interface SessionSerie {
  idx: number;
  reps: number;
  weight: number;
}

// From src-tauri/src/ui/sessions/models.rs:100
export interface SessionSeriesUpdate {
  series: SessionSerie[];
  timestamp: string;
}

// From src-tauri/src/ui/user/models.rs:7
export interface UserListItem {
  date: string;
  fat_ratio: number;
  lean_mass: number;
  water_ratio: number;
  weight: number;
}

// From src-tauri/src/ui/workouts/models.rs:33
export interface WorkoutDetails {
  avg_time: string;
  avg_volume: number;
  latest_session: string;
  name: string;
  session_count: number;
  sessions: WorkoutSession[];
}

// From src-tauri/src/ui/workouts/models.rs:6
export interface WorkoutListItem {
  avg_time: string;
  latest_session: string;
  name: string;
  sessions: number;
}

// From src-tauri/src/ui/workouts/models.rs:14
export interface WorkoutSession {
  date: string;
  time: string;
  vol_diff: string;
  volume: number;
}
