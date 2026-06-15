import { ExerciseDetails, ExerciseListItem } from "@/models/exercises";
import {
  WorkoutDetails,
  WorkoutListItem,
  WorkoutSeriesUpdate,
} from "@/models/workouts";
import { invoke } from "@tauri-apps/api/core";

export class RpcUtils {
  public static getWorkouts(): Promise<WorkoutListItem[]> {
    return invoke("get_workouts");
  }
  public static getWorkoutDetails(timestamp: number): Promise<WorkoutDetails> {
    return invoke("get_workout_details", { timestamp });
  }
  public static getExercises(): Promise<ExerciseListItem[]> {
    return invoke("get_exercises");
  }
  public static getExerciseDetails(
    category: string,
    id: number,
  ): Promise<ExerciseDetails> {
    return invoke("get_exercise_details", { category, id });
  }
  public static importFile(): Promise<void> {
    return invoke("import_file");
  }
  public static saveWorkoutChanges(workout: WorkoutSeriesUpdate) {
    return invoke("save_workout_changes", { details: workout });
  }
}
