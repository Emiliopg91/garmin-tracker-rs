import { AppContext } from "@/context/AppContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { LoadingContext } from "@/context/LoadingContext";
import { BackendClient } from "@/utils/backend/client";
import {
  SessionFrontDetails,
  SessionUtils,
  WorkoutLoad,
} from "@/utils/SessionUtils";
import { usePickerAdapter } from "@mui/x-date-pickers/hooks";
import { useContext, useEffect, useState } from "react";
import "@/styles/Home/Home.css";
import "@/styles/Sessions/SessionModal.css";
import {
  SessionListItem,
  WorkoutDetails,
  WorkoutListItem,
} from "@/utils/backend/models";
import { BackendListener } from "@/utils/backend/listener";
import { SessionModal } from "../Sessions/SessionModal";
import { WorkoutModal } from "../Workouts/WorkoutModal";
import { Heatmap } from "./helpers/Heatmap";
import { WorkloadChart } from "./helpers/WorkloadChart";
import { ImportSessionsMenu } from "./helpers/ImportSessionsMenu";
import { WorkoutSetsModal } from "../Workouts/WorkoutSetsModal";
import { WorkoutActionsMenu } from "../Workouts/helpers/WorkoutActionsMenu";

export function Home() {
  const { sessionsVersion } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, fromKg, formatDuration, formatTimeDate, formatNumber } =
    useContext(I18nSettingsContext);
  const adapter = usePickerAdapter();

  const [day, setDay] = useState(() => new Date().toDateString());
  const [availableData, setAvailableData] = useState(false);
  const [workload, setWorkload] = useState<WorkoutLoad[]>([]);
  const [minDate, setMinDate] = useState(0);

  const [todaySessions, setTodaySessions] = useState(0);
  const [todayTime, setTodayTime] = useState(0);
  const [todayKcal, setTodayKCal] = useState(0);
  const [todayLoad, setTodayLoad] = useState(0);
  const [previousWeekSessions, setPreviousWeekSessions] = useState(0);
  const [previousWeekTime, setPreviousWeekTime] = useState(0);
  const [previousWeekKcal, setPreviousWeekKCal] = useState(0);
  const [previousWeekLoad, setPreviousWeekLoad] = useState(0);
  const [thisWeekSessions, setThisWeekSessions] = useState(0);
  const [thisWeekTime, setThisWeekTime] = useState(0);
  const [thisWeekKcal, setThisWeekKCal] = useState(0);
  const [thisWeekLoad, setThisWeekLoad] = useState(0);
  const [monthSessions, setMonthSessions] = useState(0);
  const [monthTime, setMonthTime] = useState(0);
  const [monthKcal, setMonthKCal] = useState(0);
  const [monthLoad, setMonthLoad] = useState(0);
  const [heatMapData, setHeatMapData] = useState<
    [number, number][][] | undefined
  >(undefined);

  const [workout, setWorkout] = useState<WorkoutListItem | undefined>(
    undefined,
  );
  const [rest, setRest] = useState(false);
  const [workoutDetails, setWorkoutDetails] = useState<
    WorkoutDetails | undefined
  >(undefined);
  const [session, setLastSession] = useState<SessionListItem | undefined>(
    undefined,
  );
  const [sessionDetails, setSessionDetails] = useState<
    SessionFrontDetails | undefined
  >(undefined);
  const [workoutEdit, setWorkoutEdit] = useState<WorkoutDetails | undefined>(
    undefined,
  );

  const refresh = () => {
    startLoading();
    const today = new Date(new Date().setHours(0, 0, 0, 0)).getTime() / 1000;
    BackendClient.getSessions(
      today - SessionUtils.CHRONIC_DAYS * 2 * 24 * 60 * 60,
    )
      .then((sessions) => {
        setAvailableData(sessions.length > 0);

        if (sessions.length > 0) {
          const workout_data = SessionUtils.calculateWorkoutLoad(sessions);

          startLoading();
          BackendClient.getHeatmapData()
            .then((data) => {
              setHeatMapData(data);
            })
            .finally(() => {
              finishLoading();
            });

          startLoading();
          BackendClient.getWorkoutList()
            .then((workouts) => {
              const filtered = workouts.filter((w) => w.enabled);

              if (!overreaching && filtered.length > 0) {
                // Never-done workouts take priority, otherwise the least recent one
                let oldest = filtered.find((w) => w.latest_session == null);
                if (oldest === undefined) {
                  oldest = filtered[0];
                  for (const w of filtered) {
                    if (oldest.latest_session! > w.latest_session!) {
                      oldest = w;
                    }
                  }
                }
                setWorkout(oldest);
              } else {
                setWorkout(undefined);
              }
            })
            .finally(() => {
              finishLoading();
            });

          let todaySessions = 0;
          let todayTmp = 0;
          let todayKcl = 0;
          let todayLoad = 0;
          let thisWeekSessions = 0;
          let thisWeekTmp = 0;
          let thisWeekKcl = 0;
          let thisWeekLoad = 0;
          let prevWeekSessions = 0;
          let prevWeekTmp = 0;
          let prevWeekKcl = 0;
          let prevWeekLoad = 0;
          let monthSessions = 0;
          let monthTmp = 0;
          let monthKcl = 0;
          let monthLoad = 0;

          const startOfWeek = adapter.startOfWeek(
            adapter.startOfDay(new Date()),
          );

          const thisWeekLimit = startOfWeek.getTime() / 1000;
          const prevWeekLimit =
            adapter.addWeeks(startOfWeek, -1).getTime() / 1000;
          const monthLimit = today - 29 * 24 * 60 * 60;

          for (let i = 0; i < sessions.length; i++) {
            if (sessions[i].timestamp < monthLimit) {
              break;
            }
            monthSessions += 1;
            monthTmp += sessions[i].total_elapsed_time;
            monthKcl += sessions[i].active_calories;
            monthLoad += sessions[i].training_load;
            if (sessions[i].timestamp >= prevWeekLimit) {
              if (sessions[i].timestamp < thisWeekLimit) {
                prevWeekSessions += 1;
                prevWeekTmp += sessions[i].total_elapsed_time;
                prevWeekKcl += sessions[i].active_calories;
                prevWeekLoad += sessions[i].training_load;
              } else {
                thisWeekSessions += 1;
                thisWeekTmp += sessions[i].total_elapsed_time;
                thisWeekKcl += sessions[i].active_calories;
                thisWeekLoad += sessions[i].training_load;

                if (sessions[i].timestamp >= today) {
                  todaySessions += 1;
                  todayTmp += sessions[i].total_elapsed_time;
                  todayKcl += sessions[i].active_calories;
                  todayLoad += sessions[i].training_load;
                }
              }
            }
          }

          setTodaySessions(todaySessions);
          setTodayKCal(todayKcl);
          setTodayTime(todayTmp);
          setTodayLoad(todayLoad);
          setThisWeekSessions(thisWeekSessions);
          setThisWeekKCal(thisWeekKcl);
          setThisWeekTime(thisWeekTmp);
          setThisWeekLoad(thisWeekLoad);
          setPreviousWeekSessions(prevWeekSessions);
          setPreviousWeekKCal(prevWeekKcl);
          setPreviousWeekTime(prevWeekTmp);
          setPreviousWeekLoad(prevWeekLoad);
          setMonthSessions(monthSessions);
          setMonthKCal(monthKcl);
          setMonthTime(monthTmp);
          setMonthLoad(monthLoad);

          setWorkload(workout_data);
          if (workout_data.length > 0) {
            setMinDate(workout_data[0].date);
          }

          setLastSession(sessions[0]);

          const overreaching = SessionUtils.isOverreaching(workout_data);
          const lastSessDate = new Date(
            sessions[0].timestamp * 1000,
          ).toDateString();
          const todayDate = new Date().toDateString();
          setRest(todayLoad > 50 || overreaching || lastSessDate == todayDate);
        }
      })
      .finally(() => {
        finishLoading();
      });
  };

  const getSessionDetails = (timestamp: number) => {
    startLoading();
    BackendClient.getSessionDetails(timestamp)
      .then((details) => {
        setSessionDetails(SessionUtils.detailsFromBackend(details, fromKg));
      })
      .finally(() => {
        finishLoading();
      });
  };
  const getWorkoutDetails = (name: string) => {
    startLoading();
    BackendClient.getWorkoutDetails(name)
      .then((details) => {
        setWorkoutDetails(details);
      })
      .finally(() => {
        finishLoading();
      });
  };

  useEffect(() => {
    const checkDay = () => setDay(new Date().toDateString());

    const now = new Date();
    const nextMidnight = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate() + 1,
    );
    const timer = setTimeout(
      checkDay,
      nextMidnight.getTime() - now.getTime() + 1000,
    );

    // Timers may not fire on time after the system suspends
    document.addEventListener("visibilitychange", checkDay);
    window.addEventListener("focus", checkDay);

    return () => {
      clearTimeout(timer);
      document.removeEventListener("visibilitychange", checkDay);
      window.removeEventListener("focus", checkDay);
    };
  }, [day]);

  useEffect(() => {
    const unregisterSessionLocation = BackendListener.onSessionLocationUpdate(
      (data) => {
        setLastSession((prev) =>
          prev && prev.timestamp == data.session
            ? { ...prev, name: data.location }
            : prev,
        );
      },
    );

    const unregisterAddedSteps = BackendListener.onAddedWorkoutSteps(
      (names) => {
        setWorkout((prev) =>
          prev && names.includes(prev.name)
            ? { ...prev, has_steps: true }
            : prev,
        );
      },
    );

    refresh();

    return () => {
      unregisterSessionLocation();
      unregisterAddedSteps();
    };
  }, [sessionsVersion, adapter, day]);

  const openEdit = (workout: string) => {
    startLoading();
    BackendClient.getWorkoutDetails(workout)
      .then((details) => {
        setWorkoutDetails(undefined);
        setWorkoutEdit(details);
      })
      .finally(() => {
        finishLoading();
      });
  };

  return (
    <>
      {!availableData && (
        <>
          <h1 id="no-available-data">{translate("no_available_data")}</h1>
          <h3 id="import-sessions">{translate("import_session_to_begin")}</h3>
        </>
      )}
      {availableData && (
        <>
          <div className="dashboard-box">
            <fieldset>
              <legend>{translate("training_status")}</legend>
              <div style={{ display: "flex" }}>
                {workload.length > 0 && (
                  <WorkloadChart workload={workload} minDate={minDate} />
                )}
                <Heatmap data={heatMapData} />
              </div>
              <table id="last-days">
                <colgroup>
                  <col />
                  <col />
                  <col />
                  <col />
                </colgroup>
                <thead>
                  <tr>
                    <th></th>
                    <th>{translate("today")}</th>
                    <th>{translate("this_week")}</th>
                    <th>{translate("previous_week")}</th>
                    <th>{translate("last_30_days")}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td>{translate("sessions")}</td>
                    <td>{formatNumber(todaySessions, 0)}</td>
                    <td>{formatNumber(thisWeekSessions, 0)}</td>
                    <td>{formatNumber(previousWeekSessions, 0)}</td>
                    <td>{formatNumber(monthSessions, 0)}</td>
                  </tr>
                </tbody>
                <tbody>
                  <tr>
                    <td>{translate("workout_load")}</td>
                    <td>
                      {Math.round(todayLoad / todaySessions) +
                        " / " +
                        formatNumber(todayLoad, 0)}
                    </td>
                    <td>
                      {Math.round(thisWeekLoad / thisWeekSessions) +
                        " / " +
                        formatNumber(thisWeekLoad, 0)}
                    </td>
                    <td>
                      {Math.round(previousWeekLoad / previousWeekSessions) +
                        " / " +
                        formatNumber(previousWeekLoad, 0)}
                    </td>
                    <td>
                      {Math.round(monthLoad / monthSessions) +
                        " / " +
                        formatNumber(monthLoad, 0)}
                    </td>
                  </tr>
                </tbody>
                <tbody>
                  <tr>
                    <td>{translate("active_time")}</td>
                    <td>{formatDuration(todayTime)}</td>
                    <td>{formatDuration(thisWeekTime)}</td>
                    <td>{formatDuration(previousWeekTime)}</td>
                    <td>{formatDuration(monthTime)}</td>
                  </tr>
                </tbody>
                <tbody>
                  <tr>
                    <td>{translate("active_calories")}</td>
                    <td>{formatNumber(todayKcal, 0)} Kcal</td>
                    <td>{formatNumber(thisWeekKcal, 0)} Kcal</td>
                    <td>{formatNumber(previousWeekKcal, 0)} Kcal</td>
                    <td>{formatNumber(monthKcal, 0)} Kcal</td>
                  </tr>
                </tbody>
              </table>
            </fieldset>
          </div>
          <div className="dashboard-box">
            <fieldset>
              <legend>{translate("last_session")}</legend>
              <div id="list-layer">
                <table>
                  <thead>
                    <tr>
                      <th className="text-center">{translate("date")}</th>
                      <th className="text-center">{translate("sport")}</th>
                      <th className="text-center">{translate("name")}</th>
                      <th className="text-center">
                        {translate("active_calories")}
                      </th>
                      <th className="text-center">
                        {translate("workout_load")}
                      </th>
                    </tr>
                  </thead>

                  <tbody>
                    <tr
                      className="clickable-row"
                      onClick={() => getSessionDetails(session!.timestamp)}
                    >
                      <td>{formatTimeDate(session!.timestamp)}</td>
                      <td>
                        {translate("sport_" + session!.sport) +
                          " - " +
                          translate(
                            "sport_" +
                              session!.sport +
                              "_" +
                              session!.sub_sport,
                          )}
                      </td>
                      <td>{session!.name}</td>
                      <td>{formatNumber(session!.active_calories, 0)}</td>
                      <td>{formatNumber(session!.training_load, 0)}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </fieldset>
          </div>
          <div className="dashboard-box">
            <fieldset>
              <legend>{translate("activity_suggestion")}</legend>
              {rest && (
                <div style={{ textAlign: "center" }}>
                  <p>{translate("rest_recommended")}</p>
                </div>
              )}
              {!rest && workout && (
                <div id="list-layer">
                  <table>
                    <thead>
                      <tr>
                        <th className="text-center">{translate("workout")}</th>
                        <th className="text-center">
                          {translate("latest_session")}
                        </th>
                        <th className="text-center">
                          {translate("session_count")}
                        </th>
                        <th className="text-center">
                          {translate("average_duration")}
                        </th>
                        <th></th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr
                        className="clickable-row"
                        onClick={() => getWorkoutDetails(workout.name)}
                      >
                        <td className="text-left">
                          {workout.name.length > 0 && (
                            <span>{workout.name}</span>
                          )}
                          {workout.name.length == 0 && (
                            <span>{translate("other")}</span>
                          )}
                        </td>
                        <td>
                          {workout.latest_session &&
                            formatTimeDate(workout.latest_session)}
                        </td>
                        <td>{formatNumber(workout.sessions, 0)}</td>
                        <td>{formatDuration(workout.avg_time)}</td>
                        <td onClick={(e) => e.stopPropagation()}>
                          {workout.has_steps && (
                            <WorkoutActionsMenu
                              workoutName={workout.name}
                              onOpenEdit={() => openEdit(workout.name)}
                              dense
                            />
                          )}
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              )}
            </fieldset>
          </div>
        </>
      )}

      <div>
        {sessionDetails && (
          <SessionModal
            session={sessionDetails}
            onClose={() => setSessionDetails(undefined)}
            onUpdate={() => {
              refresh();
            }}
          />
        )}
      </div>

      <div>
        {workoutDetails && (
          <WorkoutModal
            workout={workoutDetails}
            showEnable={false}
            onClose={() => setWorkoutDetails(undefined)}
            onOpenEdit={() => {
              openEdit(workoutDetails.name);
            }}
          />
        )}
      </div>

      <div>
        {workoutEdit && (
          <WorkoutSetsModal
            workout={workoutEdit}
            isEdit={true}
            onClose={() => setWorkoutEdit(undefined)}
          />
        )}
      </div>

      <ImportSessionsMenu onImported={refresh} />
    </>
  );
}
