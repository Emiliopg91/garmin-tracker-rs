//Auto generated file, do not edit manually

import { invoke, InvokeArgs } from "@tauri-apps/api/core";

import { AppEnvironment, BodyMetricListItem, ExerciseDetails, ExerciseListItem, SessionDetails, SessionListItem, SessionSeriesUpdate, Settings, WorkoutDetails, WorkoutListItem } from "./models";

export class BackendClient {

    private static DONT_LOG_COMMANDS: string[] = [];

	// From src-tauri/src/logic/body_metrics.rs:67
	public static addBodyMeasures(measures: BodyMetricListItem): Promise<void> {
	  return BackendClient.inner_invoke("add_body_measures", { measures }); 
	}
	

	// From src-tauri/src/logic/body_metrics.rs:106
	public static deleteBodyMetric(date: number): Promise<void> {
	  return BackendClient.inner_invoke("delete_body_metric", { date }); 
	}
	

	// From src-tauri/src/logic/app.rs:126
	public static exportDatabase(): Promise<void> {
	  return BackendClient.inner_invoke("export_database"); 
	}
	

	// From src-tauri/src/logic/body_metrics.rs:24
	public static getBodyMeasures(): Promise<BodyMetricListItem[]> {
	  return BackendClient.inner_invoke("get_body_measures"); 
	}
	

	// From src-tauri/src/logic/app.rs:62
	public static getEnvironment(): Promise<AppEnvironment> {
	  return BackendClient.inner_invoke("get_environment"); 
	}
	

	// From src-tauri/src/logic/exercises.rs:93
	public static getExerciseDetails(category: string, id: number): Promise<ExerciseDetails> {
	  return BackendClient.inner_invoke("get_exercise_details", { category, id }); 
	}
	

	// From src-tauri/src/logic/exercises.rs:36
	public static getExercises(): Promise<ExerciseListItem[]> {
	  return BackendClient.inner_invoke("get_exercises"); 
	}
	

	// From src-tauri/src/logic/sessions.rs:83
	public static getSessionDetails(timestamp: number): Promise<SessionDetails> {
	  return BackendClient.inner_invoke("get_session_details", { timestamp }); 
	}
	

	// From src-tauri/src/logic/sessions.rs:46
	public static getSessions(): Promise<SessionListItem[]> {
	  return BackendClient.inner_invoke("get_sessions"); 
	}
	

	// From src-tauri/src/logic/app.rs:29
	public static getSettings(): Promise<Settings> {
	  return BackendClient.inner_invoke("get_settings"); 
	}
	

	// From src-tauri/src/logic/app.rs:177
	public static getTranslations(): Promise<Record<string, string>> {
	  return BackendClient.inner_invoke("get_translations"); 
	}
	

	// From src-tauri/src/logic/workouts.rs:91
	public static getWorkoutDetails(name: string): Promise<WorkoutDetails> {
	  return BackendClient.inner_invoke("get_workout_details", { name }); 
	}
	

	// From src-tauri/src/logic/workouts.rs:30
	public static getWorkoutList(): Promise<WorkoutListItem[]> {
	  return BackendClient.inner_invoke("get_workout_list"); 
	}
	

	// From src-tauri/src/logic/sessions.rs:221
	public static importFromDevice(serial: string): Promise<number> {
	  return BackendClient.inner_invoke("import_from_device", { serial }); 
	}
	

	// From src-tauri/src/logic/app.rs:36
	public static notifyFrontendReady(): Promise<void> {
	  return BackendClient.inner_invoke("notify_frontend_ready"); 
	}
	

	// From src-tauri/src/logic/sessions.rs:163
	public static saveSessionChanges(details: SessionSeriesUpdate): Promise<void> {
	  return BackendClient.inner_invoke("save_session_changes", { details }); 
	}
	

	// From src-tauri/src/logic/app.rs:73
	public static updateSettingsValue(name: string, value: string): Promise<void> {
	  return BackendClient.inner_invoke("update_settings_value", { name, value }); 
	}
	

  
	private static inner_invoke<R>(method: string, payload?: InvokeArgs): Promise<R> {
		return new Promise<R>((resolve,reject)=>{
			const do_log = !BackendClient.DONT_LOG_COMMANDS.includes(method);
			if(do_log) {
				console.debug("Invoking command '"+method+"', payload: ", payload);
			}
			invoke<R>(method, payload).then((response)=>{
				if(do_log) {
					console.debug("Finished command '"+method+"', response: ", response);
				}
				resolve(response);
			}).catch((err) =>{
				if(do_log) {
					console.debug("Failed command '"+method+"', reason: ", err);
				}
				reject(err);
			});
		});
	}
}