//Auto generated file, do not edit manually

import { invoke, InvokeArgs } from "@tauri-apps/api/core";

import {
  AppEnvironment,
  ExerciseDetails,
  ExerciseListItem,
  SessionDetails,
  SessionListItem,
  SessionSeriesUpdate,
  UserListItem,
  WorkoutDetails,
  WorkoutListItem,
} from "./models";

export class BackendClient {
  // Definition: /ui/user/mod.rs:43
  public static addUserMeasures(measures: UserListItem): Promise<void> {
    return BackendClient.inner_invoke("add_user_measures", { measures });
  }

  // Definition: /ui/app/mod.rs:33
  public static getEnvironment(): Promise<AppEnvironment> {
    return BackendClient.inner_invoke("get_environment");
  }

  // Definition: /ui/exercises/mod.rs:58
  public static getExerciseDetails(
    category: string,
    id: number,
  ): Promise<ExerciseDetails> {
    return BackendClient.inner_invoke("get_exercise_details", { category, id });
  }

  // Definition: /ui/exercises/mod.rs:16
  public static getExercises(): Promise<ExerciseListItem[]> {
    return BackendClient.inner_invoke("get_exercises");
  }

  // Definition: /ui/sessions/mod.rs:59
  public static getSessionDetails(timestamp: string): Promise<SessionDetails> {
    return BackendClient.inner_invoke("get_session_details", { timestamp });
  }

  // Definition: /ui/sessions/mod.rs:28
  public static getSessions(): Promise<SessionListItem[]> {
    return BackendClient.inner_invoke("get_sessions");
  }

  // Definition: /ui/user/mod.rs:15
  public static getUserMeasures(): Promise<UserListItem[]> {
    return BackendClient.inner_invoke("get_user_measures");
  }

  // Definition: /ui/workouts/mod.rs:77
  public static getWorkoutDetails(name: string): Promise<WorkoutDetails> {
    return BackendClient.inner_invoke("get_workout_details", { name });
  }

  // Definition: /ui/workouts/mod.rs:17
  public static getWorkoutList(): Promise<WorkoutListItem[]> {
    return BackendClient.inner_invoke("get_workout_list");
  }

  // Definition: /ui/sessions/mod.rs:225
  public static importFromDevice(serial: string): Promise<number> {
    return BackendClient.inner_invoke("import_from_device", { serial });
  }

  // Definition: /ui/sessions/mod.rs:168
  public static importFromFile(): Promise<number> {
    return BackendClient.inner_invoke("import_from_file");
  }

  // Definition: /ui/app/mod.rs:21
  public static notifyFrontendReady(): Promise<void> {
    return BackendClient.inner_invoke("notify_frontend_ready");
  }

  // Definition: /ui/app/mod.rs:116
  public static openVersionChangelog(version: string): Promise<void> {
    return BackendClient.inner_invoke("open_version_changelog", { version });
  }

  // Definition: /ui/sessions/mod.rs:104
  public static saveSessionChanges(
    details: SessionSeriesUpdate,
  ): Promise<void> {
    return BackendClient.inner_invoke("save_session_changes", { details });
  }

  private static inner_invoke<R>(
    method: string,
    payload?: InvokeArgs,
  ): Promise<R> {
    return invoke(method, payload);
  }
}
