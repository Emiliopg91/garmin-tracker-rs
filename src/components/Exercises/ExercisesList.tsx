import { ExerciseDetails, ExerciseListItem } from "@/utils/backend/models";
import { ExerciseModal } from "./ExerciseModal";
import { useContext, useEffect, useState } from "react";
import { BackendClient } from "@/utils/backend/client";
import { AppContext } from "@/context/AppContext";
import { LoadingContext } from "@/context/LoadingContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";

export function ExercisesList() {
  const { sessionsVersion } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);
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
  }, [sessionsVersion]);

  const getExerciseDetails = (category: number, id: number) => {
    BackendClient.getExerciseDetails(category, id).then((details) => {
      setExerciseDetails(details);
    });
  };

  return (
    <>
      <table>
        <colgroup>
          <col className="col-400" />
          <col className="col-100" />
          <col className="col-60" />
          <col className="col-100" />
        </colgroup>
        <thead>
          <tr>
            <th className="text-center">{translate("exercise")}</th>
            <th className="text-center">{translate("pr")}</th>
            <th className="text-center">{translate("rm")}</th>
            <th className="text-center">{translate("date")}</th>
          </tr>
        </thead>
        <tbody>
          {exercises.map((exercise, idx) => (
            <tr
              key={idx}
              className="clickable-row"
              onClick={() => getExerciseDetails(exercise.category, exercise.id)}
            >
              <td className="text-left">
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
                {UnitUtils.fromKg(exercise.e1rm, settings.weight_unit) +
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
