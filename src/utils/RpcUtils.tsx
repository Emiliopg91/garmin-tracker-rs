import { ExerciseDetails, ExerciseListItem } from "@/models/exercises";
import { WorkoutDetails, WorkoutListItem } from "@/models/workouts";
import { invoke } from "@tauri-apps/api/core";

export class RpcUtils {
  public static getWorkouts(): Promise<WorkoutListItem[]> {
    return invoke("get_workouts");
  }
  public static getWorkoutDetails(timestamp: string): Promise<WorkoutDetails> {
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
}
