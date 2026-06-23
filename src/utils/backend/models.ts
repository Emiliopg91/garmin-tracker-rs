// Definition: /garmin/models/devices.rs:5
export interface DeviceListItem {
  manufacturer: string;
  model: string;
  serial_number: string;
}

// Definition: /garmin/models/exercises.rs:33
export interface ExerciseDetails {
  name: string;
  reps: number;
  weight: number;
  rm: number;
  pr_date: string;
  id: number;
  series: Record<string, SessionSerie[]>;
  workouts: string[];
  category: string;
}

// Definition: /garmin/models/exercises.rs:8
export interface ExerciseListItem {
  category: string;
  rm: number;
  name: string;
  weight: number;
  date: string;
  id: number;
  reps: number;
}

// Definition: /garmin/models/notifications.rs:4
export interface NotificationDefinition {
  body: string;
  title: string;
}

// Definition: /garmin/models/sessions.rs:59
export interface SessionDetails {
  metabolic_calories: number;
  date: string;
  total_calories: number;
  active_time: string;
  series: Record<string, SessionSerie[]>;
  exercises: string[];
  total_elapsed_time: string;
  max_heart_rate: number;
  timestamp: string;
  avg_heart_rate: number;
  name: string;
}

// Definition: /garmin/models/sessions.rs:8
export interface SessionListItem {
  name: string;
  exercises_num: number;
  timestamp: string;
  date: string;
  volume: number;
  series_num: number;
}

// Definition: /garmin/models/sessions.rs:42
export interface SessionSerie {
  weight: number;
  idx: number;
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
  sessions: WorkoutSession[];
  name: string;
  avg_time: string;
  avg_volume: number;
  session_count: number;
}

// Definition: /garmin/models/workouts.rs:6
export interface WorkoutListItem {
  sessions: number;
  name: string;
  latest_session: string;
  avg_time: string;
}

// Definition: /garmin/models/workouts.rs:14
export interface WorkoutSession {
  volume: number;
  date: string;
  time: string;
  vol_diff: string;
}

