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
  private static DONT_LOG_COMMANDS: string[] = [];

  // From src-tauri/src/ui/user/mod.rs:50
  public static addUserMeasures(measures: UserListItem): Promise<void> {
    return BackendClient.inner_invoke("add_user_measures", { measures });
  }

  // From src-tauri/src/ui/app/mod.rs:55
  public static getEnvironment(): Promise<AppEnvironment> {
    return BackendClient.inner_invoke("get_environment");
  }

  // From src-tauri/src/ui/exercises/mod.rs:72
  public static getExerciseDetails(
    category: string,
    id: number,
  ): Promise<ExerciseDetails> {
    return BackendClient.inner_invoke("get_exercise_details", { category, id });
  }

  // From src-tauri/src/ui/exercises/mod.rs:28
  public static getExercises(): Promise<ExerciseListItem[]> {
    return BackendClient.inner_invoke("get_exercises");
  }

  // From src-tauri/src/ui/sessions/mod.rs:69
  public static getSessionDetails(timestamp: string): Promise<SessionDetails> {
    return BackendClient.inner_invoke("get_session_details", { timestamp });
  }

  // From src-tauri/src/ui/sessions/mod.rs:39
  public static getSessions(): Promise<SessionListItem[]> {
    return BackendClient.inner_invoke("get_sessions");
  }

  // From src-tauri/src/ui/user/mod.rs:19
  public static getUserMeasures(): Promise<UserListItem[]> {
    return BackendClient.inner_invoke("get_user_measures");
  }

  // From src-tauri/src/ui/workouts/mod.rs:76
  public static getWorkoutDetails(name: string): Promise<WorkoutDetails> {
    return BackendClient.inner_invoke("get_workout_details", { name });
  }

  // From src-tauri/src/ui/workouts/mod.rs:21
  public static getWorkoutList(): Promise<WorkoutListItem[]> {
    return BackendClient.inner_invoke("get_workout_list");
  }

  // From src-tauri/src/ui/sessions/mod.rs:207
  public static importFromDevice(serial: string): Promise<number> {
    return BackendClient.inner_invoke("import_from_device", { serial });
  }

  // From src-tauri/src/ui/sessions/mod.rs:178
  public static importFromFile(): Promise<number> {
    return BackendClient.inner_invoke("import_from_file");
  }

  // From src-tauri/src/ui/app/mod.rs:32
  public static notifyFrontendReady(): Promise<void> {
    return BackendClient.inner_invoke("notify_frontend_ready");
  }

  // From src-tauri/src/ui/app/mod.rs:144
  public static openVersionChangelog(version: string): Promise<void> {
    return BackendClient.inner_invoke("open_version_changelog", { version });
  }

  // From src-tauri/src/ui/sessions/mod.rs:113
  public static saveSessionChanges(
    details: SessionSeriesUpdate,
  ): Promise<void> {
    return BackendClient.inner_invoke("save_session_changes", { details });
  }

  private static inner_invoke<R>(
    method: string,
    payload?: InvokeArgs,
  ): Promise<R> {
    return new Promise<R>((resolve, reject) => {
      const do_log = !BackendClient.DONT_LOG_COMMANDS.includes(method);
      if (do_log) {
        console.debug("Invoking command '" + method + "', payload: ", payload);
      }
      invoke<R>(method, payload)
        .then((response) => {
          if (do_log) {
            console.debug(
              "Finished command '" + method + "', response: ",
              response,
            );
          }
          resolve(response);
        })
        .catch((err) => {
          if (do_log) {
            console.debug("Failed command '" + method + "', reason: ", err);
          }
          reject(err);
        });
    });
  }
}
