import { WorkoutSerie } from "./workouts";

export interface ExerciseListItem {
  name: string;
  category: string;
  id: number;
  reps: number;
  weight: number;
  rm: number;
}

export interface ExerciseDetails {
  category: string;
  id: number;
  name: string;
  reps: number;
  weight: number;
  rm: number;
  workouts: Record<string, WorkoutSerie[]>;
}
