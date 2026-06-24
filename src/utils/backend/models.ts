// Definition: /ui/devices/models.rs:5
export interface DeviceListItem {
  manufacturer: string;
  model: string;
  serial_number: string;
}

// Definition: /ui/exercises/models.rs:33
export interface ExerciseDetails {
  pr_date: string;
  weight: number;
  series: Record<string, SessionSerie[]>;
  workouts: string[];
  rm: number;
  name: string;
  id: number;
  reps: number;
  category: string;
}

// Definition: /ui/exercises/models.rs:8
export interface ExerciseListItem {
  category: string;
  name: string;
  rm: number;
  date: string;
  reps: number;
  id: number;
  weight: number;
}

// Definition: /ui/sessions/models.rs:59
export interface SessionDetails {
  timestamp: string;
  max_heart_rate: number;
  series: Record<string, SessionSerie[]>;
  exercises: string[];
  metabolic_calories: number;
  total_elapsed_time: string;
  total_calories: number;
  date: string;
  active_time: string;
  avg_heart_rate: number;
  name: string;
}

// Definition: /ui/sessions/models.rs:8
export interface SessionListItem {
  date: string;
  volume: number;
  series_num: number;
  exercises_num: number;
  name: string;
  timestamp: string;
}

// Definition: /ui/sessions/models.rs:42
export interface SessionSerie {
  idx: number;
  weight: number;
  reps: number;
}

// Definition: /ui/sessions/models.rs:96
export interface SessionSeriesUpdate {
  series: SessionSerie[];
  timestamp: string;
}

// Definition: /ui/workouts/models.rs:33
export interface WorkoutDetails {
  sessions: WorkoutSession[];
  latest_session: string;
  name: string;
  avg_time: string;
  session_count: number;
  avg_volume: number;
}

// Definition: /ui/workouts/models.rs:6
export interface WorkoutListItem {
  latest_session: string;
  name: string;
  sessions: number;
  avg_time: string;
}

// Definition: /ui/workouts/models.rs:14
export interface WorkoutSession {
  volume: number;
  vol_diff: string;
  time: string;
  date: string;
}

