// Definition: /ui/devices/models.rs:5
export interface DeviceListItem {
  manufacturer: string;
  model: string;
  serial_number: string;
}

// Definition: /ui/exercises/models.rs:33
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

// Definition: /ui/exercises/models.rs:8
export interface ExerciseListItem {
  category: string;
  date: string;
  id: number;
  name: string;
  reps: number;
  rm: number;
  weight: number;
}

// Definition: /ui/sessions/models.rs:59
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

// Definition: /ui/sessions/models.rs:8
export interface SessionListItem {
  date: string;
  exercises_num: number;
  name: string;
  series_num: number;
  timestamp: string;
  volume: number;
}

// Definition: /ui/sessions/models.rs:42
export interface SessionSerie {
  idx: number;
  reps: number;
  weight: number;
}

// Definition: /ui/sessions/models.rs:96
export interface SessionSeriesUpdate {
  series: SessionSerie[];
  timestamp: string;
}

// Definition: /ui/user/models.rs:7
export interface UserListItem {
  date: string;
  fat_ratio: number;
  lean_mass: number;
  water_ratio: number;
  weight: number;
}

// Definition: /ui/workouts/models.rs:33
export interface WorkoutDetails {
  avg_time: string;
  avg_volume: number;
  latest_session: string;
  name: string;
  session_count: number;
  sessions: WorkoutSession[];
}

// Definition: /ui/workouts/models.rs:6
export interface WorkoutListItem {
  avg_time: string;
  latest_session: string;
  name: string;
  sessions: number;
}

// Definition: /ui/workouts/models.rs:14
export interface WorkoutSession {
  date: string;
  time: string;
  vol_diff: string;
  volume: number;
}

// Definition: /ui/app/models.rs:4
export enum AppEnvironment {
	Debug = "Debug",
	Release = "Release",
}

