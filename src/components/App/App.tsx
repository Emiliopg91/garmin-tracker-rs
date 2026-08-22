import { AppContext } from "@/context/AppContext";
import { JSX, useContext } from "react";
import { NavBar } from "../NavBar/NavBar";
import "@/styles/app.css";
import { Tabs } from "@/models/tabs";
import { SessionsList } from "../Sessions/SessionList";
import { ExercisesList } from "../Exercises/ExercisesList";
import { BodyMetricList } from "../BodyMetrics/BodyMetricList";
import { Loading } from "../Loading/Loading";
import { WorkoutsList } from "../Workouts/WorkoutList";
import { Settings } from "../Settings/Settings";

export function App(): JSX.Element {
  const { tab, setTab, loading, appReady, translate } = useContext(AppContext);

  const leftNavBarItems = [
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
      label: translate("body_metrics"),
      onSelected: () => {
        setTab(Tabs.BODY_METRICS);
      },
      selected: tab == Tabs.BODY_METRICS,
    },
  ];

  const rightNavBarItems = [
    {
      label: translate("settings"),
      onSelected: () => {
        setTab(Tabs.SETTINGS);
      },
      selected: tab == Tabs.SETTINGS,
    },
  ];

  return (
    <>
      <div id="viewport">
        {!appReady || (loading && <Loading />)}

        {appReady && (
          <>
            <NavBar leftItems={leftNavBarItems} rightItems={rightNavBarItems} />

            <div id="list-layer">
              {tab == Tabs.SESSIONS && <SessionsList />}
              {tab == Tabs.EXERCISES && <ExercisesList />}
              {tab == Tabs.WORKOUTS && <WorkoutsList />}
              {tab == Tabs.BODY_METRICS && <BodyMetricList />}
              {tab == Tabs.SETTINGS && <Settings />}
            </div>
          </>
        )}
      </div>
    </>
  );
}

export default App;
