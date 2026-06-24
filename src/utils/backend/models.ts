// Definition: /ui/devices/models.rs:5
export interface DeviceListItem {
  serial_number: string;
  manufacturer: string;
  model: string;
}

// Definition: /ui/exercises/models.rs:33
export interface ExerciseDetails {
  rm: number;
  workouts: string[];
  weight: number;
  name: string;
  reps: number;
  category: string;
  id: number;
  pr_date: string;
  series: Record<string, SessionSerie[]>;
}

// Definition: /ui/exercises/models.rs:8
export interface ExerciseListItem {
  date: string;
  category: string;
  rm: number;
  id: number;
  name: string;
  reps: number;
  weight: number;
}

// Definition: /ui/sessions/models.rs:59
export interface SessionDetails {
  date: string;
  timestamp: string;
  total_elapsed_time: string;
  active_time: string;
  name: string;
  avg_heart_rate: number;
  metabolic_calories: number;
  max_heart_rate: number;
  series: Record<string, SessionSerie[]>;
  exercises: string[];
  total_calories: number;
}

// Definition: /ui/sessions/models.rs:8
export interface SessionListItem {
  name: string;
  volume: number;
  series_num: number;
  exercises_num: number;
  timestamp: string;
  date: string;
}

// Definition: /ui/sessions/models.rs:42
export interface SessionSerie {
  reps: number;
  weight: number;
  idx: number;
}

// Definition: /ui/sessions/models.rs:96
export interface SessionSeriesUpdate {
  timestamp: string;
  series: SessionSerie[];
}

// Definition: /ui/workouts/models.rs:33
export interface WorkoutDetails {
  latest_session: string;
  avg_time: string;
  avg_volume: number;
  sessions: WorkoutSession[];
  session_count: number;
  name: string;
}

// Definition: /ui/workouts/models.rs:6
export interface WorkoutListItem {
  sessions: number;
  name: string;
  avg_time: string;
  latest_session: string;
}

// Definition: /ui/workouts/models.rs:14
export interface WorkoutSession {
  time: string;
  vol_diff: string;
  date: string;
  volume: number;
}

