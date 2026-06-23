import { AppContext } from "@/context/AppContext";
import { JSX, useContext } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";
import { Tabs } from "@/models/tabs";
import { SessionsList } from "../Sessions/SessionList";
import { ExercisesList } from "../Exercises/ExercisesList";
import { WorkoutsList } from "../Workouts/WorkoutList";
import { Loading } from "../Loading/Loading";

export function App(): JSX.Element {
  const { tab, setTab, loading } = useContext(AppContext);

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
        {loading && <Loading />}

        <NavBar items={navBarItems} />

        <div id="list-layer">
          {tab == Tabs.SESSIONS && <SessionsList />}
          {tab == Tabs.EXERCISES && <ExercisesList />}
          {tab == Tabs.WORKOUTS && <WorkoutsList />}
        </div>
      </div>
    </>
  );
}

export default App;
