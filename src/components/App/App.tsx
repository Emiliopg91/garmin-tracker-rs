import { AppContext } from "@/context/AppContext";
import { JSX, useContext } from "react";
import { NavBar, NavBarItem } from "../NavBar/NavBar";
import "@/styles/app.css";
import { Tabs } from "@/models/tabs";
import { SessionsList } from "../Sessions/SessionList";
import { ExercisesList } from "../Exercises/ExercisesList";
import { BodyMetricList } from "../BodyMetrics/BodyMetricList";
import { Loading } from "../Loading/Loading";
import { WorkoutsList } from "../Workouts/WorkoutList";
import { Settings } from "../Settings/Settings";
import SettingsIcon from "@mui/icons-material/Settings";

export function App(): JSX.Element {
  const {
    tab,
    setTab,
    loading,
    appReady,
    translate,
    showSettings,
    settingsOpened,
    closeSettings,
  } = useContext(AppContext);

  const leftNavBarItems: NavBarItem[] = [
    {
      label: <span>{translate("sessions")}</span>,
      onSelected: () => {
        setTab(Tabs.SESSIONS);
      },
      selected: tab == Tabs.SESSIONS,
    },
    {
      label: <span>{translate("workouts")}</span>,
      onSelected: () => {
        setTab(Tabs.WORKOUTS);
      },
      selected: tab == Tabs.WORKOUTS,
    },
    {
      label: <span>{translate("exercises")}</span>,
      onSelected: () => {
        setTab(Tabs.EXERCISES);
      },
      selected: tab == Tabs.EXERCISES,
    },
    {
      label: <span>{translate("body_metrics")}</span>,
      onSelected: () => {
        setTab(Tabs.BODY_METRICS);
      },
      selected: tab == Tabs.BODY_METRICS,
    },
  ];

  const rightNavBarItems: NavBarItem[] = [
    {
      label: (
        <span>
          <SettingsIcon />
        </span>
      ),
      onSelected: () => {
        showSettings();
      },
      selected: false,
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
              {settingsOpened && <Settings onClose={closeSettings} />}
            </div>
          </>
        )}
      </div>
    </>
  );
}

export default App;
