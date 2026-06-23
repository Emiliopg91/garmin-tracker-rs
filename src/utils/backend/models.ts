// Definition: /garmin/models/devices.rs:5
export interface DeviceListItem {
  model: string;
  manufacturer: string;
  serial_number: string;
}

// Definition: /garmin/models/exercises.rs:33
export interface ExerciseDetails {
  workouts: string[];
  name: string;
  rm: number;
  id: number;
  reps: number;
  pr_date: string;
  category: string;
  series: Record<string, SessionSerie[]>;
  weight: number;
}

// Definition: /garmin/models/exercises.rs:8
export interface ExerciseListItem {
  name: string;
  date: string;
  rm: number;
  category: string;
  id: number;
  reps: number;
  weight: number;
}

// Definition: /garmin/models/notifications.rs:4
export interface NotificationDefinition {
  body: string;
  title: string;
}

// Definition: /garmin/models/sessions.rs:59
export interface SessionDetails {
  total_elapsed_time: string;
  exercises: string[];
  name: string;
  metabolic_calories: number;
  active_time: string;
  timestamp: string;
  avg_heart_rate: number;
  max_heart_rate: number;
  date: string;
  total_calories: number;
  series: Record<string, SessionSerie[]>;
}

// Definition: /garmin/models/sessions.rs:8
export interface SessionListItem {
  series_num: number;
  date: string;
  volume: number;
  name: string;
  timestamp: string;
  exercises_num: number;
}

// Definition: /garmin/models/sessions.rs:42
export interface SessionSerie {
  idx: number;
  weight: number;
  reps: number;
}

// Definition: /garmin/models/sessions.rs:96
export interface SessionSeriesUpdate {
  timestamp: string;
  series: SessionSerie[];
}

// Definition: /garmin/models/workouts.rs:33
export interface WorkoutDetails {
  latest_session: string;
  avg_time: string;
  session_count: number;
  name: string;
  avg_volume: number;
  sessions: WorkoutSession[];
}

// Definition: /garmin/models/workouts.rs:6
export interface WorkoutListItem {
  avg_time: string;
  latest_session: string;
  sessions: number;
  name: string;
}

// Definition: /garmin/models/workouts.rs:14
export interface WorkoutSession {
  volume: number;
  time: string;
  vol_diff: string;
  date: string;
}

