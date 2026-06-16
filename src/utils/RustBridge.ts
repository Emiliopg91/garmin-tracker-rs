//Auto generated file, do not edit manually

import { invoke } from "@tauri-apps/api/core";

export interface ExerciseListItem {
	category: string
	id: number
	name: string
	reps: number
	weight: number
	rm: number
}

export interface ExerciseDetails {
	category: string
	id: number
	name: string
	reps: number
	weight: number
	rm: number
	workouts: string[]
	series: Record<string, WorkoutSerie[]>
	pr_date: string
}

export interface WorkoutSerie {
	idx: number
	reps: number
	weight: number
}

export interface WorkoutSeriesUpdate {
	timestamp: string
	series: WorkoutSerie[]
}

export interface WorkoutDetails {
	name: string
	date: string
	timestamp: string
	total_elapsed_time: string
	active_time: string
	total_calories: number
	metabolic_calories: number
	avg_heart_rate: number
	max_heart_rate: number
	exercises: string[]
	series: Record<string, WorkoutSerie[]>
}

export interface WorkoutListItem {
	name: string
	date: string
	timestamp: string
}

export class RustBridge {

	public static getWorkouts(): Promise<WorkoutListItem[]> {
		return invoke("get_workouts");
	}


	public static getWorkoutDetails(timestamp: string): Promise<WorkoutDetails> {
		return invoke("get_workout_details", { timestamp });
	}


	public static getExercises(): Promise<ExerciseListItem[]> {
		return invoke("get_exercises");
	}


	public static getExerciseDetails(category: string, id: number): Promise<ExerciseDetails> {
		return invoke("get_exercise_details", { category, id });
	}


	public static importFile(): Promise<void> {
		return invoke("import_file");
	}


	public static saveWorkoutChanges(details: WorkoutSeriesUpdate): Promise<void> {
		return invoke("save_workout_changes", { details });
	}

}