import { WorkoutDetails, WorkoutListItem } from "@/utils/backend/models";
import { WorkoutModal } from "./WorkoutModal";
import { BackendClient } from "@/utils/backend/client";
import { useContext, useEffect, useState } from "react";
import { AppContext } from "@/context/AppContext";

export function WorkoutsList() {
  const { setLoading } = useContext(AppContext);

  const [workouts, setWorkouts] = useState<WorkoutListItem[]>([]);
  const [workoutDetails, setWorkoutDetails] = useState<
    WorkoutDetails | undefined
  >(undefined);

  const refreshList = () => {
    BackendClient.getWorkoutList()
      .then((data) => {
        setWorkouts(data);
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error getting workout list",
          body: e,
        });
      })
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    refreshList();
  }, []);

  const getWorkoutDetails = (name: string) => {
    BackendClient.getWorkoutDetails(name).then((details) => {
      setWorkoutDetails(details);
    });
  };
  return (
    <>
      <table>
        <thead>
          <tr>
            <th style={{ textAlign: "center" }}>Workout</th>
            <th style={{ textAlign: "center" }}>Latest session</th>
            <th style={{ textAlign: "center" }}>Session count</th>
            <th style={{ textAlign: "center" }}>Average duration</th>
          </tr>
        </thead>
        <tbody>
          {workouts.map((workout, idx) => (
            <tr
              key={idx}
              style={{ cursor: "pointer" }}
              onClick={() => getWorkoutDetails(workout.name)}
            >
              <td style={{ textAlign: "left" }}>{workout.name}</td>
              <td>{workout.latest_session}</td>
              <td>{workout.sessions}</td>
              <td>{workout.avg_time}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div>
        {workoutDetails && (
          <WorkoutModal
            workout={workoutDetails}
            onClose={() => setWorkoutDetails(undefined)}
          />
        )}
      </div>
    </>
  );
}
