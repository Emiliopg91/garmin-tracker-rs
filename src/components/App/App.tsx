import { AppContext } from "@/context/AppContext";
import { JSX, useContext, useEffect, useState } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";
import { RpcUtils } from "@/utils/RpcUtils";
import { Tabs } from "@/models/tabs";
import { WorkoutDetails, WorkoutListItem } from "@/models/workouts";
import { ExerciseDetails, ExerciseListItem } from "@/models/exercises";
import { Button } from "react-bootstrap";
import { WorkoutsList } from "../Workouts/WorkoutList";
import { ExercisesList } from "../Exercises/ExercisesList";
import { WorkoutModal } from "../Workouts/WorkoutModal";
import { ExerciseModal } from "../Exercises/ExerciseModal";

export function App(): JSX.Element {
  const { tab, setTab } = useContext(AppContext);

  const [workouts, setWorkouts] = useState<WorkoutListItem[]>([]);
  const [workoutDetails, setWorkoutDetails] = useState<
    WorkoutDetails | undefined
  >(undefined);

  const [exercises, setExercises] = useState<ExerciseListItem[]>([]);
  const [exerciseDetails, setExerciseDetails] = useState<
    ExerciseDetails | undefined
  >(undefined);

  useEffect(() => {
    RpcUtils.getWorkouts().then((data) => {
      setWorkouts(data);

      RpcUtils.getExercises().then((data) => {
        setExercises(data);
      });
    });
  }, []);

  const navBarItems = [
    {
      label: "Workouts",
      onSelected: () => {
        setTab(Tabs.WORKOUTS);
      },
      selected: tab == Tabs.WORKOUTS,
    },
    {
      label: "Exercises",
      onSelected: () => {
        setTab(Tabs.EXERCISES);
      },
      selected: tab == Tabs.EXERCISES,
    },
  ];

  const getWorkoutDetails = (timestamp: number) => {
    RpcUtils.getWorkoutDetails(timestamp).then((details) => {
      setWorkoutDetails(details);
    });
  };

  const getExerciseDetails = (category: string, id: number) => {
    RpcUtils.getExerciseDetails(category, id).then((details) => {
      setExerciseDetails(details);
    });
  };

  const importFile = () => {
    RpcUtils.importFile().then(() => {
      RpcUtils.getWorkouts().then((data) => {
        setWorkouts(data);

        RpcUtils.getExercises().then((data) => {
          setExercises(data);
        });
      });
    });
  };

  return (
    <>
      <div id="viewport">
        <NavBar items={navBarItems} />

        <div id="list-layer">
          {tab == Tabs.WORKOUTS && (
            <WorkoutsList
              workouts={workouts}
              onRowClick={(timestamp) => {
                getWorkoutDetails(timestamp);
              }}
            />
          )}
          {tab == Tabs.EXERCISES && (
            <ExercisesList
              exercises={exercises}
              onRowClick={(category, id) => {
                getExerciseDetails(category, id);
              }}
            />
          )}
        </div>
        <div style={{ padding: "5px" }}>
          <Button id="import-button" onClick={importFile}>
            Import .fit file
          </Button>
        </div>

        <div>
          {workoutDetails && (
            <WorkoutModal
              workout={workoutDetails}
              onClose={() => setWorkoutDetails(undefined)}
            />
          )}
        </div>

        <div>
          {exerciseDetails && (
            <ExerciseModal
              exercise={exerciseDetails}
              onClose={() => setExerciseDetails(undefined)}
            />
          )}
        </div>
      </div>
    </>
  );
}

export default App;
