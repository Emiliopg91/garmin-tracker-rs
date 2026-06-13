import { ExerciseListItem } from "@/models/exercises";
import { RecordListItem } from "@/models/records";
import { WorkoutListItem } from "@/models/workouts";
import { invoke } from "@tauri-apps/api/core";

export class RpcUtils {
  public static getWorkouts(): Promise<WorkoutListItem[]> {
    return invoke("get_workouts");
  }
  public static getExercises(): Promise<ExerciseListItem[]> {
    return invoke("get_exercises");
  }
  public static getRecords(): Promise<RecordListItem[]> {
    return invoke("get_records");
  }
}
