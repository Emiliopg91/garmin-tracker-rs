import {
  StepType,
  WorkoutDetails,
  WorkoutListItem,
} from "@/utils/backend/models";
import { WorkoutModal } from "./WorkoutModal";
import { WorkoutRow } from "./helpers/WorkoutRow";
import { BackendClient } from "@/utils/backend/client";
import { useContext, useEffect, useState } from "react";
import { AppContext } from "@/context/AppContext";
import { LoadingContext } from "@/context/LoadingContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import "@/styles/Workouts/WorkoutLists.css";
import { WorkoutSetsModal } from "./WorkoutSetsModal";
import { Button } from "@mui/material";

export function WorkoutsList() {
  const { sessionsVersion } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate } = useContext(I18nSettingsContext);

  const [workouts, setWorkouts] = useState<WorkoutListItem[]>([]);
  const [workoutDetails, setWorkoutDetails] = useState<
    WorkoutDetails | undefined
  >(undefined);
  const [workoutEdit, setWorkoutEdit] = useState<WorkoutDetails | undefined>(
    undefined,
  );
  const [isEdit, setIsEdit] = useState(false);

  const openEdit = (workout: WorkoutDetails | undefined) => {
    if (workout === undefined) {
      setIsEdit(false);
      const workout: WorkoutDetails = {
        name: "",
        avg_time: 0,
        avg_volume: 0,
        enabled: true,
        latest_session: 0,
        session_count: 0,
        sessions: [],
        steps: [
          {
            idx: 0,
            workout: "",
            kind: StepType.Rest,
            time: 120000,
            begin_idx: null,
            ex_cat: null,
            ex_id: null,
            reps: null,
            weight: null,
          },
          {
            idx: 1,
            workout: "",
            kind: StepType.Repeat,
            begin_idx: 0,
            time: null,
            ex_cat: null,
            ex_id: null,
            reps: 1,
            weight: null,
          },
        ],
      };
      setWorkoutEdit(workout);
    } else {
      setIsEdit(true);
      setWorkoutEdit(workout);
    }
    setWorkoutDetails(undefined);
  };

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
      <div id="list-layer">
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
      </div>

      <div>
        {workoutDetails && (
          <WorkoutModal
            workout={workoutDetails}
            showEnable={true}
            onClose={() => setWorkoutDetails(undefined)}
            onUpdate={setWorkoutState}
            onOpenEdit={openEdit}
          />
        )}
      </div>

      <div>
        {workoutEdit && (
          <WorkoutSetsModal
            workout={workoutEdit}
            isEdit={isEdit}
            onClose={() => {
              setWorkoutEdit(undefined);
              refreshList();
            }}
          />
        )}
      </div>

      <div className="list-action-bar">
        <Button
          className="full-width-button"
          onClick={() => openEdit(undefined)}
          variant="contained"
        >
          {translate("create_workout")}
        </Button>
      </div>
    </>
  );
}
