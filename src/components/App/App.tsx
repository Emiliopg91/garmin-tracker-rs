import { AppContext } from "@/context/AppContext";
import { JSX, useContext, useEffect, useState } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";
import {
  ExerciseDetails,
  ExerciseListItem,
  RustBridge,
  SessionDetails,
  SessionListItem,
  WorkoutDetails,
  WorkoutListItem,
} from "@/utils/RustBridge";
import { Tabs } from "@/models/tabs";
import { Button } from "react-bootstrap";
import { SessionsList } from "../Sessions/SessionList";
import { ExercisesList } from "../Exercises/ExercisesList";
import { SessionModal } from "../Sessions/SessionModal";
import { ExerciseModal } from "../Exercises/ExerciseModal";
import { Alerts } from "../Alerts/Alerts";
import { AlertType } from "@/models/alert";
import { WorkoutsList } from "../Workouts/WorkoutList";
import { WorkoutModal } from "../Workouts/WorkoutModal";

export function App(): JSX.Element {
  const { tab, setTab, alerts, addAlert } = useContext(AppContext);

  const [sessions, setSessions] = useState<SessionListItem[]>([]);
  const [sessionDetails, setSessionDetails] = useState<
    SessionDetails | undefined
  >(undefined);

  const [exercises, setExercises] = useState<ExerciseListItem[]>([]);
  const [exerciseDetails, setExerciseDetails] = useState<
    ExerciseDetails | undefined
  >(undefined);

  const [workouts, setWorkouts] = useState<WorkoutListItem[]>([]);
  const [workoutDetails, setWorkoutDetails] = useState<
    WorkoutDetails | undefined
  >(undefined);

  const refreshLists = () => {
    RustBridge.getSessions()
      .then((data) => {
        setSessions(data);
      })
      .catch((e) => {
        addAlert({
          title: "Error getting session list",
          body: e,
          type: AlertType.ERROR,
        });
      })
      .finally(() => {
        RustBridge.getWorkoutList()
          .then((data) => {
            setWorkouts(data);
          })
          .catch((e) => {
            addAlert({
              title: "Error getting workout list",
              body: e,
              type: AlertType.ERROR,
            });
          })
          .finally(() => {
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
          });
      });
  };

  useEffect(() => {
    refreshLists();
  }, []);

  const getSessionDetails = (timestamp: string) => {
    RustBridge.getSessionDetails(timestamp)
      .then((details) => {
        setSessionDetails(details);
      })
      .catch((e) => {
        addAlert({
          title: "Error getting session details",
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

  const getWorkoutDetails = (name: string) => {
    RustBridge.getWorkoutDetails(name).then((details) => {
      setWorkoutDetails(details);
    });
  };

  const importFile = () => {
    RustBridge.importFile()
      .then((session) => {
        refreshLists();
        addAlert({
          title: "File imported succesfully",
          body: "Imported session '" + session.name + "' from " + session.date,
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
      label: "Sessions",
      onSelected: () => {
        setTab(Tabs.SESSIONS);
      },
      selected: tab == Tabs.SESSIONS,
    },
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
          {tab == Tabs.SESSIONS && (
            <SessionsList
              sessions={sessions}
              onRowClick={(timestamp) => {
                getSessionDetails(timestamp);
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
          {tab == Tabs.WORKOUTS && (
            <WorkoutsList workouts={workouts} onRowClick={getWorkoutDetails} />
          )}
        </div>
        <div style={{ padding: "5px" }}>
          <Button id="import-button" onClick={importFile}>
            Import .fit file
          </Button>
        </div>

        <div>
          {sessionDetails && (
            <SessionModal
              session={sessionDetails}
              onClose={() => setSessionDetails(undefined)}
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

        <div>
          {workoutDetails && (
            <WorkoutModal
              workout={workoutDetails}
              onClose={() => setWorkoutDetails(undefined)}
            />
          )}
        </div>
      </div>
    </>
  );
}

export default App;
