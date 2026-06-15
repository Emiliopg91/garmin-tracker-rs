export interface WorkoutListItem {
  name: string;
  date: string;
  timestamp: number;
}

export interface WorkoutDetails {
  name: string;
  date: string;
  total_elapsed_time: string;
  active_time: string;
  total_calories: number;
  metabolic_calories: number;
  avg_heart_rate: number;
  max_heart_rate: number;
  volume: number;
  series: Record<string, WorkoutSerie[]>;
}

export interface WorkoutSerie {
  name: string;
  reps: number;
  weight: number;
}
