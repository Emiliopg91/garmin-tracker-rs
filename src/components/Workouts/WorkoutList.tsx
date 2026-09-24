import { WorkoutDetails, WorkoutListItem } from "@/utils/backend/models";
import { WorkoutModal } from "./WorkoutModal";
import { WorkoutRow } from "./helpers/WorkoutRow";
import { BackendClient } from "@/utils/backend/client";
import { useContext, useEffect, useState } from "react";
import { AppContext } from "@/context/AppContext";
import { LoadingContext } from "@/context/LoadingContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import "@/styles/Workouts/WorkoutLists.css";

export function WorkoutsList() {
  const { sessionsVersion } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate } = useContext(I18nSettingsContext);

  const [workouts, setWorkouts] = useState<WorkoutListItem[]>([]);
  const [workoutDetails, setWorkoutDetails] = useState<
    WorkoutDetails | undefined
  >(undefined);

  const refreshList = () => {
    startLoading();
    BackendClient.getWorkoutList()
      .then((data) => {
        data.sort((a, b) => {
          if (a.name.length > 0 && b.name.length > 0) {
            return a.name.localeCompare(b.name);
          } else {
            if (a.name.length == 0) {
              return 1;
            } else {
              return -1;
            }
          }
        });
        setWorkouts(data);
      })
      .finally(() => {
        finishLoading();
      });
  };

  useEffect(() => {
    refreshList();
  }, [sessionsVersion]);

  const getWorkoutDetails = (name: string) => {
    BackendClient.getWorkoutDetails(name).then((details) => {
      setWorkoutDetails(details);
    });
  };

  const setWorkoutState = (name: string, status: boolean) => {
    const res = workouts.map((w) => {
      if (w.name == name) {
        return { ...w, enabled: status };
      } else {
        return { ...w };
      }
    });
    setWorkouts(res);
  };

  return (
    <>
      <table>
        <thead>
          <tr>
            <th className="text-center">{translate("workout")}</th>
            <th className="text-center">{translate("latest_session")}</th>
            <th className="text-center">{translate("session_count")}</th>
            <th className="text-center">{translate("average_duration")}</th>
          </tr>
        </thead>
        <tbody>
          {workouts.map((workout, idx) => (
            <WorkoutRow
              key={idx}
              workout={workout}
              onSelect={getWorkoutDetails}
            />
          ))}
        </tbody>
      </table>

      <div>
        {workoutDetails && (
          <WorkoutModal
            workout={workoutDetails}
            showEnable={true}
            onClose={() => setWorkoutDetails(undefined)}
            onUpdate={setWorkoutState}
          />
        )}
      </div>
    </>
  );
}
