import { StepType, WorkoutStep } from "./backend/models";

export type LeafStep = Omit<WorkoutStep, "begin_idx" | "idx" | "workout">;

export interface StepGroup {
  laps: number;
  wrapped: LeafStep[];
}

export type WorkoutStepData = LeafStep | StepGroup;

export class WorkoutUtils {
  public static isStepGroup(step: WorkoutStepData): step is StepGroup {
    return "wrapped" in step;
  }

  public static parseWorkoutSteps(steps: WorkoutStep[]): WorkoutStepData[] {
    const elements: WorkoutStepData[] = [];

    let localSteps = [...steps].sort((a, b) => a.idx - b.idx);

    while (localSteps.length > 0) {
      const elem = localSteps[localSteps.length - 1];

      if (elem.kind == StepType.Repeat) {
        const laps = elem.reps!;
        const wrapped = localSteps
          .filter((ws) => ws.idx >= elem.begin_idx! && ws.idx < elem.idx)
          .map(WorkoutUtils.workoutStepToLeafStep);
        const element: StepGroup = {
          laps,
          wrapped,
        };
        elements.push(element);
        localSteps = localSteps.filter((ws) => ws.idx < elem.begin_idx!);
      } else {
        elements.push(WorkoutUtils.workoutStepToLeafStep(elem));
        localSteps.pop();
      }
    }

    return elements.reverse();
  }

  private static workoutStepToLeafStep(step: WorkoutStep): LeafStep {
    if (step.kind == StepType.Rest) {
      return {
        kind: StepType.Rest,
        time:
          step.time !== null && step.time !== undefined
            ? step.time / 1000
            : null,
        reps: step.reps !== null && step.reps !== undefined ? step.reps : null,
        ex_cat: null,
        ex_id: null,
        weight: null,
      };
    } else {
      return {
        kind: StepType.Exercise,
        time:
          step.time !== null && step.time !== undefined
            ? step.time / 1000
            : null,
        reps: step.reps !== null && step.reps !== undefined ? step.reps : null,
        ex_cat: step.ex_cat,
        ex_id: step.ex_id,
        weight: step.weight,
      };
    }
  }
}
