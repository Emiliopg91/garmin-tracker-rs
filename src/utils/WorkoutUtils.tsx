import { StepType, WeightUnit, WorkoutStep } from "./backend/models";
import { UnitUtils } from "./UnitUtils";

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

  public static parseWorkoutSteps(
    steps: WorkoutStep[],
    weightUnit: WeightUnit,
  ): WorkoutStepData[] {
    const elements: WorkoutStepData[] = [];

    let localSteps = [...steps].sort((a, b) => a.idx - b.idx);

    while (localSteps.length > 0) {
      const elem = localSteps[localSteps.length - 1];

      if (elem.kind == StepType.Repeat) {
        const laps = elem.reps!;
        const wrapped = localSteps
          .filter((ws) => ws.idx >= elem.begin_idx! && ws.idx < elem.idx)
          .map((e) => WorkoutUtils.workoutStepToLeafStep(e, weightUnit));
        const element: StepGroup = {
          laps,
          wrapped,
        };
        elements.push(element);
        localSteps = localSteps.filter((ws) => ws.idx < elem.begin_idx!);
      } else {
        elements.push(WorkoutUtils.workoutStepToLeafStep(elem, weightUnit));
        localSteps.pop();
      }
    }

    return elements.reverse();
  }

  private static workoutStepToLeafStep(
    step: WorkoutStep,
    weightUnit: WeightUnit,
  ): LeafStep {
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
        weight:
          step.weight != null
            ? Math.round(UnitUtils.fromKg(step.weight, weightUnit) * 10) / 10
            : null,
      };
    }
  }

  public static validateSteps(steps: WorkoutStepData[]): boolean {
    return (
      steps.length > 0 &&
      steps.every((step) =>
        WorkoutUtils.isStepGroup(step)
          ? step.laps >= 1 &&
            step.wrapped.length > 0 &&
            step.wrapped.every(WorkoutUtils.validateLeaf)
          : WorkoutUtils.validateLeaf(step),
      )
    );
  }

  // Leftover fields on rest steps are not checked here, they're dropped before sending to the backend
  private static validateLeaf(leaf: LeafStep): boolean {
    if (leaf.time != null && leaf.time <= 0) {
      return false;
    }
    if (leaf.kind == StepType.Rest) {
      return true;
    }
    return (
      leaf.ex_cat != null &&
      leaf.ex_id != null &&
      (leaf.weight == null || leaf.weight >= 0) &&
      (leaf.reps == null || leaf.reps >= 1)
    );
  }

  public static toWorkoutSteps(
    steps: WorkoutStepData[],
    workout: string,
    weightUnit: WeightUnit,
  ): WorkoutStep[] {
    const result: WorkoutStep[] = [];

    for (const step of steps) {
      if (WorkoutUtils.isStepGroup(step)) {
        const begin = result.length;
        for (const leaf of step.wrapped) {
          result.push(
            WorkoutUtils.leafToWorkoutStep(
              leaf,
              workout,
              result.length,
              weightUnit,
            ),
          );
        }
        result.push({
          workout,
          idx: result.length,
          kind: StepType.Repeat,
          reps: step.laps,
          begin_idx: begin,
          ex_cat: null,
          ex_id: null,
          time: null,
          weight: null,
        });
      } else {
        result.push(
          WorkoutUtils.leafToWorkoutStep(
            step,
            workout,
            result.length,
            weightUnit,
          ),
        );
      }
    }

    return result;
  }

  private static leafToWorkoutStep(
    leaf: LeafStep,
    workout: string,
    idx: number,
    weightUnit: WeightUnit,
  ): WorkoutStep {
    if (leaf.kind == StepType.Rest) {
      return {
        workout,
        idx: idx,
        kind: StepType.Rest,
        time: leaf.time != null ? leaf.time * 1000 : null,
        begin_idx: null,
        ex_cat: null,
        ex_id: null,
        reps: null,
        weight: null,
      };
    } else {
      return {
        workout,
        idx: idx,
        kind: StepType.Exercise,
        time: leaf.time != null ? leaf.time * 1000 : null,
        ex_cat: leaf.ex_cat,
        ex_id: leaf.ex_id,
        reps: leaf.reps,
        weight:
          leaf.weight != null
            ? Math.round(UnitUtils.toKg(leaf.weight, weightUnit) * 10) / 10
            : null,
        begin_idx: null,
      };
    }
  }

  public static DEFAULT_REST: LeafStep = {
    kind: StepType.Rest,
    time: 120,
    ex_cat: null,
    ex_id: null,
    reps: null,
    weight: null,
  };

  public static DEFAULT_EXERCISE: LeafStep = {
    kind: StepType.Exercise,
    time: null,
    ex_cat: null,
    ex_id: null,
    reps: 1,
    weight: 0,
  };

  public static DEFAULT_GROUP: StepGroup = {
    laps: 3,
    wrapped: [
      { ...WorkoutUtils.DEFAULT_EXERCISE },
      { ...WorkoutUtils.DEFAULT_REST },
    ],
  };
}
