import { ExerciseDetails, ExerciseListItem } from "@/utils/backend/models";
import { ExerciseModal } from "./ExerciseModal";
import { useContext, useEffect, useState } from "react";
import { BackendClient } from "@/utils/backend/client";
import { AppContext } from "@/context/AppContext";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";

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
        setExercises(data);
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
                {UnitUtils.fromKg(exercise.rm, settings.weight_unit).toFixed(
                  1,
                ) +
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
