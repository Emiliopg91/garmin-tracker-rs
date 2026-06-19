/* eslint-disable */

//Auto generated file, do not edit manually

import { invoke, InvokeArgs } from "@tauri-apps/api/core";

import { SessionListItem, SessionDetails, ExerciseListItem, ExerciseDetails, SessionSeriesUpdate, WorkoutListItem, WorkoutDetails, DeviceListItem, NotificationDefinition } from "./models";

export class BackendClient {

	// Definition: src-tauri/src/lib.rs:63
	public static getAvailableDevices(): Promise<DeviceListItem[]> {
		return BackendClient.inner_invoke("get_available_devices");
	}

	// Definition: src-tauri/src/lib.rs:33
	public static getExerciseDetails(category: string, id: number): Promise<ExerciseDetails> {
		return BackendClient.inner_invoke("get_exercise_details", { category, id });
	}

	// Definition: src-tauri/src/lib.rs:28
	public static getExercises(): Promise<ExerciseListItem[]> {
		return BackendClient.inner_invoke("get_exercises");
	}

	// Definition: src-tauri/src/lib.rs:23
	public static getSessionDetails(timestamp: string): Promise<SessionDetails> {
		return BackendClient.inner_invoke("get_session_details", { timestamp });
	}

	// Definition: src-tauri/src/lib.rs:18
	public static getSessions(): Promise<SessionListItem[]> {
		return BackendClient.inner_invoke("get_sessions");
	}

	// Definition: src-tauri/src/lib.rs:58
	public static getWorkoutDetails(name: string): Promise<WorkoutDetails> {
		return BackendClient.inner_invoke("get_workout_details", { name });
	}

	// Definition: src-tauri/src/lib.rs:53
	public static getWorkoutList(): Promise<WorkoutListItem[]> {
		return BackendClient.inner_invoke("get_workout_list");
	}

	// Definition: src-tauri/src/lib.rs:43
	public static importFromDevice(serial: string): Promise<number> {
		return BackendClient.inner_invoke("import_from_device", { serial });
	}

	// Definition: src-tauri/src/lib.rs:38
	public static importFromFile(): Promise<number> {
		return BackendClient.inner_invoke("import_from_file");
	}

	// Definition: src-tauri/src/lib.rs:48
	public static saveSessionChanges(details: SessionSeriesUpdate): Promise<void> {
		return BackendClient.inner_invoke("save_session_changes", { details });
	}

	// Definition: src-tauri/src/lib.rs:68
	public static showNotification(notification: NotificationDefinition): Promise<void> {
		return BackendClient.inner_invoke("show_notification", { notification });
	}

	private static inner_invoke<R>(method: string, payload?: InvokeArgs): Promise<R> {
		return invoke(method, payload);
	}
}