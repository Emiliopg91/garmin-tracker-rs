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

export function App(): JSX.Element {
  const { tab, setTab, loading, appReady } = useContext(AppContext);

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
    {
      label: "User",
      onSelected: () => {
        setTab(Tabs.USER);
      },
      selected: tab == Tabs.USER,
    },
  ];

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
          </>
        )}
      </div>
    </>
  );
}

export default App;
