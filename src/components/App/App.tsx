import { AppContext } from "@/context/AppContext";
import { JSX, useContext, useEffect, useRef, useState } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";
import { Tabs } from "@/models/tabs";
import { Dropdown } from "react-bootstrap";
import { SessionsList } from "../Sessions/SessionList";
import { ExercisesList } from "../Exercises/ExercisesList";
import { SessionModal } from "../Sessions/SessionModal";
import { ExerciseModal } from "../Exercises/ExerciseModal";
import { WorkoutsList } from "../Workouts/WorkoutList";
import { WorkoutModal } from "../Workouts/WorkoutModal";
import {
  DeviceListItem,
  ExerciseDetails,
  ExerciseListItem,
  SessionDetails,
  SessionListItem,
  WorkoutDetails,
  WorkoutListItem,
} from "@/utils/backend/models";
import { BackendClient } from "@/utils/backend/client";

export function App(): JSX.Element {
  const { tab, setTab } = useContext(AppContext);

  const [availableDevices, setAvailableDevices] = useState<DeviceListItem[]>(
    [],
  );

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
    BackendClient.getSessions()
      .then((data) => {
        setSessions(data);
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error getting session list",
          body: e,
        });
      })
      .finally(() => {
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
            BackendClient.getExercises()
              .then((data) => {
                setExercises(data);
              })
              .catch((e) => {
                BackendClient.showNotification({
                  title: "Error getting exercise list",
                  body: e,
                });
              });
          });
      });
  };

  const availableDevicesRef = useRef<DeviceListItem[]>([]);

  useEffect(() => {
    const interval = setInterval(() => {
      BackendClient.getAvailableDevices().then((devices) => {
        const previous = availableDevicesRef.current;

        devices.forEach((device) => {
          if (!previous.some((d) => d.serial_number === device.serial_number)) {
            BackendClient.showNotification({
              title: "Device available",
              body: device.manufacturer + " " + device.model,
            });
          }
        });

        availableDevicesRef.current = devices;
        setAvailableDevices(devices);
      });
    }, 1000);

    refreshLists();

    return () => clearInterval(interval);
  }, []);

  const getSessionDetails = (timestamp: string) => {
    BackendClient.getSessionDetails(timestamp)
      .then((details) => {
        setSessionDetails(details);
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error getting session details",
          body: e,
        });
      });
  };

  const getExerciseDetails = (category: string, id: number) => {
    BackendClient.getExerciseDetails(category, id).then((details) => {
      setExerciseDetails(details);
    });
  };

  const getWorkoutDetails = (name: string) => {
    BackendClient.getWorkoutDetails(name).then((details) => {
      setWorkoutDetails(details);
    });
  };

  const importFile = () => {
    BackendClient.importFromFile()
      .then((count) => {
        BackendClient.showNotification({
          title: "File imported succesfully",
          body: "Imported " + count + " sessions from file",
        });
        refreshLists();
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error on file import",
          body: e,
        });
      });
  };

  const importDevice = (serial: string) => {
    BackendClient.importFromDevice(serial)
      .then((count) => {
        refreshLists();
        BackendClient.showNotification({
          title: "Imported succesfully from device",
          body: "Imported " + count + " sessions from device",
        });
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error on file import",
          body: e,
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
        <div style={{ padding: "5px", width: "100%" }}>
          <Dropdown id="import-file-dropdown" className="w-100">
            <Dropdown.Toggle id="import-file-toggle">
              Import sessions
            </Dropdown.Toggle>

            <Dropdown.Menu id="import-file-menu">
              <Dropdown.Item key={"file"} onClick={importFile}>
                From file
              </Dropdown.Item>
              {availableDevices.length > 0 &&
                availableDevices.map((device, idx) => (
                  <Dropdown.Item
                    key={"dev-" + idx}
                    onClick={() => {
                      importDevice(device.serial_number);
                    }}
                  >
                    From {device.manufacturer + " " + device.model}
                  </Dropdown.Item>
                ))}
              {availableDevices.length == 0 && (
                <Dropdown.Item disabled={true}>No device found</Dropdown.Item>
              )}
            </Dropdown.Menu>
          </Dropdown>
        </div>

        <div>
          {sessionDetails && (
            <SessionModal
              session={sessionDetails}
              onClose={() => setSessionDetails(undefined)}
              onUpdate={() => refreshLists()}
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
