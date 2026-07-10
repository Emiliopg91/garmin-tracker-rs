import { AppContext } from "@/context/AppContext";
import { JSX, useContext } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";
import { Tabs } from "@/models/tabs";
import { SessionsList } from "../Sessions/SessionList";
import { ExercisesList } from "../Exercises/ExercisesList";
import { UserList } from "../User/UserList";
import { Loading } from "../Loading/Loading";
import { WorkoutsList } from "../Workouts/WorkoutList";
import { BackendClient } from "@/utils/backend/client";
import { Nav } from "react-bootstrap";

export function App(): JSX.Element {
  const { tab, setTab, loading, appReady, availableUpdate, translate } =
    useContext(AppContext);

  const navBarItems = [
    {
      label: translate("sessions"),
      onSelected: () => {
        setTab(Tabs.SESSIONS);
      },
      selected: tab == Tabs.SESSIONS,
    },
    {
      label: translate("workouts"),
      onSelected: () => {
        setTab(Tabs.WORKOUTS);
      },
      selected: tab == Tabs.WORKOUTS,
    },
    {
      label: translate("exercises"),
      onSelected: () => {
        setTab(Tabs.EXERCISES);
      },
      selected: tab == Tabs.EXERCISES,
    },
    {
      label: translate("user"),
      onSelected: () => {
        setTab(Tabs.USER);
      },
      selected: tab == Tabs.USER,
    },
  ];

  const openChangelog = () => {
    BackendClient.openVersionChangelog(availableUpdate!);
  };

  return (
    <>
      <div id="viewport">
        {!appReady || (loading && <Loading />)}

        {appReady && (
          <>
            <NavBar items={navBarItems} />

            <div id="list-layer">
              {tab == Tabs.SESSIONS && <SessionsList />}
              {tab == Tabs.EXERCISES && <ExercisesList />}
              {tab == Tabs.WORKOUTS && <WorkoutsList />}
              {tab == Tabs.USER && <UserList />}
            </div>
            {availableUpdate && (
              <div style={{ margin: "auto" }}>
                <Nav.Link onClick={openChangelog}>
                  Update {availableUpdate} available. View changes
                </Nav.Link>
              </div>
            )}
          </>
        )}
      </div>
    </>
  );
}

export default App;
