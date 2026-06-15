export interface WorkoutListItem {
  name: string;
  date: string;
  timestamp: number;
}

export interface WorkoutDetails {
  name: string;
  date: string;
  timestamp: number;
  total_elapsed_time: string;
  active_time: string;
  total_calories: number;
  metabolic_calories: number;
  avg_heart_rate: number;
  max_heart_rate: number;
  exercises: string[];
  series: Record<string, WorkoutSerie[]>;
}

export interface WorkoutSerie {
  idx: number;
  reps: number;
  weight: number;
}

export interface WorkoutSeriesUpdate {
  timestamp: number;
  series: WorkoutSerie[];
}
