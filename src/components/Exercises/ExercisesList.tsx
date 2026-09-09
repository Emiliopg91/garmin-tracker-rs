import { ExerciseDetails, ExerciseListItem } from "@/utils/backend/models";
import { ExerciseModal } from "./ExerciseModal";
import { useContext, useEffect, useState } from "react";
import { BackendClient } from "@/utils/backend/client";
import { AppContext } from "@/context/AppContext";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";

export const calc1RMEstimation = (reps: number, weight: number): number => {
  if (reps <= 1) return weight;

  if (reps <= 10 && 37 - reps > 0) {
    return (weight * 36) / (37 - reps);
  }

  if (reps <= 20) {
    if (101.3 - 2.67123 * reps > 0) {
      return (100 * weight) / (101.3 - 2.67123 * reps);
    } else {
      return weight * (1 + reps / 30);
    }
  }

  return weight * Math.pow(reps, 0.1);
};

export function ExercisesList() {
  const { startLoading, finishLoading, translate, settings } =
    useContext(AppContext);
  const [exercises, setExercises] = useState<ExerciseListItem[]>([]);
  const [exerciseDetails, setExerciseDetails] = useState<
    ExerciseDetails | undefined
  >(undefined);

  const refreshList = () => {
    startLoading();
    BackendClient.getExercises()
      .then((data) => {
        const sortedData = [...data].sort((a, b) => {
          return translate("exercise_" + a.category + "_" + a.id).localeCompare(
            translate("exercise_" + b.category + "_" + b.id),
          );
        });
        setExercises(sortedData);
      })
      .finally(() => {
        finishLoading();
      });
  };

  useEffect(() => {
    refreshList();
  }, []);

  const getExerciseDetails = (category: number, id: number) => {
    BackendClient.getExerciseDetails(category, id).then((details) => {
      setExerciseDetails(details);
    });
  };

  return (
    <>
      <table>
        <colgroup>
          <col style={{ width: "400px" }} />
          <col style={{ width: "100px" }} />
          <col style={{ width: "60px" }} />
          <col style={{ width: "100px" }} />
        </colgroup>
        <thead>
          <tr>
            <th style={{ textAlign: "center" }}>{translate("exercise")}</th>
            <th style={{ textAlign: "center" }}>{translate("pr")}</th>
            <th style={{ textAlign: "center" }}>{translate("rm")}</th>
            <th style={{ textAlign: "center" }}>{translate("date")}</th>
          </tr>
        </thead>
        <tbody>
          {exercises.map((exercise, idx) => (
            <tr
              key={idx}
              style={{ cursor: "pointer" }}
              onClick={() => getExerciseDetails(exercise.category, exercise.id)}
            >
              <td style={{ textAlign: "left" }}>
                {translate("exercise_" + exercise.category + "_" + exercise.id)}
              </td>
              <td>
                {exercise.reps +
                  "x" +
                  UnitUtils.fromKg(
                    exercise.weight,
                    settings.weight_unit,
                  ).toFixed(1) +
                  " " +
                  UnitUtils.getUnit(settings.weight_unit)}
              </td>
              <td>
                {UnitUtils.fromKg(
                  calc1RMEstimation(exercise.reps, exercise.weight),
                  settings.weight_unit,
                ).toFixed(1) +
                  " " +
                  UnitUtils.getUnit(settings.weight_unit)}
              </td>
              <td>{TimeUtils.formatDate(exercise.date)}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div>
        {exerciseDetails && (
          <ExerciseModal
            exercise={exerciseDetails}
            onClose={() => setExerciseDetails(undefined)}
          />
        )}
      </div>
    </>
  );
}
