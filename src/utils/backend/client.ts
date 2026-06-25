//Auto generated file, do not edit manually

import { invoke, InvokeArgs } from "@tauri-apps/api/core";

import { WorkoutListItem, ExerciseDetails, SessionSeriesUpdate, SessionDetails, ExerciseListItem, SessionListItem, WorkoutDetails } from "./models";

export class BackendClient {
	// Definition: /ui/exercises/mod.rs:58
	public static getExerciseDetails(category: string, id: number): Promise<ExerciseDetails> {
	  return BackendClient.inner_invoke("get_exercise_details", { category, id }); 
	}
	

	// Definition: /ui/exercises/mod.rs:16
	public static getExercises(): Promise<ExerciseListItem[]> {
	  return BackendClient.inner_invoke("get_exercises"); 
	}
	

	// Definition: /ui/sessions/mod.rs:58
	public static getSessionDetails(timestamp: string): Promise<SessionDetails> {
	  return BackendClient.inner_invoke("get_session_details", { timestamp }); 
	}
	

	// Definition: /ui/sessions/mod.rs:27
	public static getSessions(): Promise<SessionListItem[]> {
	  return BackendClient.inner_invoke("get_sessions"); 
	}
	

	// Definition: /ui/workouts/mod.rs:77
	public static getWorkoutDetails(name: string): Promise<WorkoutDetails> {
	  return BackendClient.inner_invoke("get_workout_details", { name }); 
	}
	

	// Definition: /ui/workouts/mod.rs:17
	public static getWorkoutList(): Promise<WorkoutListItem[]> {
	  return BackendClient.inner_invoke("get_workout_list"); 
	}
	

	// Definition: /ui/sessions/mod.rs:226
	public static importFromDevice(serial: string): Promise<void> {
	  return BackendClient.inner_invoke("import_from_device", { serial }); 
	}
	

	// Definition: /ui/sessions/mod.rs:167
	public static importFromFile(): Promise<void> {
	  return BackendClient.inner_invoke("import_from_file"); 
	}
	

	// Definition: /ui/sessions/mod.rs:103
	public static saveSessionChanges(details: SessionSeriesUpdate): Promise<void> {
	  return BackendClient.inner_invoke("save_session_changes", { details }); 
	}
	

	// Definition: /ui/devices/mod.rs:15
	public static startDeviceWatcher(): Promise<void> {
	  return BackendClient.inner_invoke("start_device_watcher"); 
	}
	

	private static inner_invoke<R>(method: string, payload?: InvokeArgs): Promise<R> {
		return invoke(method, payload);
	}
}