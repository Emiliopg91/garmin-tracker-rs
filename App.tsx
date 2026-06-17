import { AppContext } from "@/context/AppContext";
import { JSX, useContext, useEffect, useState } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";
import {
  ExerciseDetails,
  ExerciseListItem,
  RustBridge,
  WorkoutDetails,
  WorkoutListItem,
} from "@/utils/RustBridge";
import { Tabs } from "@/models/tabs";
import { Button } from "react-bootstrap";
import { WorkoutsList } from "../Workouts/WorkoutList";
import { ExercisesList } from "../Exercises/ExercisesList";
import { WorkoutModal } from "../Workouts/WorkoutModal";
import { ExerciseModal } from "../Exercises/ExerciseModal";
import { Alerts } from "../Alerts/Alerts";
import { AlertType } from "@/models/alert";

export function App(): JSX.Element {
  const { tab, setTab, alerts, addAlert } = useContext(AppContext);

  const [workouts, setWorkouts] = useState<WorkoutListItem[]>([]);
  const [workoutDetails, setWorkoutDetails] = useState<
    WorkoutDetails | undefined
  >(undefined);

  const [exercises, setExercises] = useState<ExerciseListItem[]>([]);
  const [exerciseDetails, setExerciseDetails] = useState<
    ExerciseDetails | undefined
  >(undefined);

  const refreshLists = () => {
    RustBridge.getWorkouts()
      .then((data) => {
        setWorkouts(data);

        RustBridge.getExercises()
          .then((data) => {
            setExercises(data);
          })
          .catch((e) => {
            addAlert({
              title: "Error getting exercise list",
              body: e,
              type: AlertType.ERROR,
            });
          });
      })
      .catch((e) => {
        addAlert({
          title: "Error getting workout list",
          body: e,
          type: AlertType.ERROR,
        });
      });
  };

  useEffect(() => {
    refreshLists();
  }, []);

  const getWorkoutDetails = (timestamp: number) => {
    RustBridge.getWorkoutDetails(timestamp)
      .then((details) => {
        setWorkoutDetails(details);
      })
      .catch((e) => {
        addAlert({
          title: "Error getting workout details",
          body: e,
          type: AlertType.ERROR,
        });
      });
  };

  const getExerciseDetails = (category: string, id: number) => {
    RustBridge.getExerciseDetails(category, id).then((details) => {
      setExerciseDetails(details);
    });
  };

  const importFile = () => {
    RustBridge.importFile()
      .then((session) => {
        refreshLists();
        addAlert({
          title: "File imported succesfully",
          body: "Imported workout '" + session.name + "' from " + session.date,
          type: AlertType.INFO,
        });
      })
      .catch((e) => {
        addAlert({
          title: "Error on file import",
          body: e,
          type: AlertType.ERROR,
        });
      });
  };

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

  return (
    <>
      <div id="viewport">
        <Alerts alerts={alerts} />

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
