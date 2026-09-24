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
import { TimeUtils } from "@/utils/TimeUtils";
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

export function Home() {
  const { availableDevices, sessionsVersion } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);
  const adapter = usePickerAdapter();

  const [day, setDay] = useState(() => new Date().toDateString());
  const [availableData, setAvailableData] = useState(false);
  const [workload, setWorkload] = useState<WorkoutLoad[]>([]);
  const [minDate, setMinDate] = useState(0);

  const [todayTime, setTodayTime] = useState(0);
  const [todayKcal, setTodayKCal] = useState(0);
  const [todayLoad, setTodayLoad] = useState(0);
  const [weekTime, setWeekTime] = useState(0);
  const [weekKcal, setWeekKCal] = useState(0);
  const [weekLoad, setWeekLoad] = useState(0);
  const [thisWeekTime, setThisWeekTime] = useState(0);
  const [thisWeekKcal, setThisWeekKCal] = useState(0);
  const [thisWeekLoad, setThisWeekLoad] = useState(0);
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
          const overreaching = SessionUtils.isOverreaching(workout_data);
          const lastSessDate = new Date(
            sessions[0].timestamp * 1000,
          ).toDateString();
          const todayDate = new Date().toDateString();
          setRest(overreaching || lastSessDate == todayDate);

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
                let oldest = filtered[0];
                for (let i = 0; i < filtered.length; i++) {
                  if (oldest.latest_session > filtered[i].latest_session) {
                    oldest = filtered[i];
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

          let todayTmp = 0;
          let todayKcl = 0;
          let todayLoad = 0;
          let thisWeekTmp = 0;
          let thisWeekKcl = 0;
          let thisWeekLoad = 0;
          let weekTmp = 0;
          let weekKcl = 0;
          let weekLoad = 0;
          let monthTmp = 0;
          let monthKcl = 0;
          let monthLoad = 0;

          const startOfWeek = adapter.startOfWeek(
            adapter.startOfDay(new Date()),
          );

          const thisWeekLimit = startOfWeek.getTime() / 1000;
          const weekLimit = today - 6 * 24 * 60 * 60;
          const monthLimit = today - 29 * 24 * 60 * 60;

          for (let i = 0; i < sessions.length; i++) {
            if (sessions[i].timestamp < monthLimit) {
              break;
            }
            monthTmp += sessions[i].total_elapsed_time;
            monthKcl += sessions[i].active_calories;
            monthLoad += sessions[i].training_load;
            if (sessions[i].timestamp >= weekLimit) {
              weekTmp += sessions[i].total_elapsed_time;
              weekKcl += sessions[i].active_calories;
              weekLoad += sessions[i].training_load;

              if (sessions[i].timestamp >= thisWeekLimit) {
                thisWeekTmp += sessions[i].total_elapsed_time;
                thisWeekKcl += sessions[i].active_calories;
                thisWeekLoad += sessions[i].training_load;

                if (sessions[i].timestamp >= today) {
                  todayTmp += sessions[i].total_elapsed_time;
                  todayKcl += sessions[i].active_calories;
                  todayLoad += sessions[i].training_load;
                }
              }
            }
          }
          setTodayKCal(todayKcl);
          setTodayTime(todayTmp);
          setTodayLoad(todayLoad);
          setThisWeekKCal(thisWeekKcl);
          setThisWeekTime(thisWeekTmp);
          setThisWeekLoad(thisWeekLoad);
          setWeekKCal(weekKcl);
          setWeekTime(weekTmp);
          setWeekLoad(weekLoad);
          setMonthKCal(monthKcl);
          setMonthTime(monthTmp);
          setMonthLoad(monthLoad);

          setWorkload(workout_data);
          if (workout_data.length > 0) {
            setMinDate(workout_data[0].date);
          }

          setLastSession(sessions[0]);
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
        setSessionDetails(
          SessionUtils.detailsFromBackend(details, settings.weight_unit),
        );
      })
      .finally(() => {
        finishLoading();
      });
  };
  const getWorkoutDetails = (name: string) => {
    BackendClient.getWorkoutDetails(name).then((details) => {
      setWorkoutDetails(details);
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

    refresh();

    return () => {
      unregisterSessionLocation();
    };
  }, [sessionsVersion, adapter, day]);

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
                    <th>{translate("last_7_days")}</th>
                    <th>{translate("last_30_days")}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td>{translate("workout_load")}</td>
                    <td>{todayLoad}</td>
                    <td>{thisWeekLoad}</td>
                    <td>{weekLoad}</td>
                    <td>{monthLoad}</td>
                  </tr>
                </tbody>
                <tbody>
                  <tr>
                    <td>{translate("active_time")}</td>
                    <td>{TimeUtils.formatDuration(todayTime)}</td>
                    <td>{TimeUtils.formatDuration(thisWeekTime)}</td>
                    <td>{TimeUtils.formatDuration(weekTime)}</td>
                    <td>{TimeUtils.formatDuration(monthTime)}</td>
                  </tr>
                </tbody>
                <tbody>
                  <tr>
                    <td>{translate("active_calories")}</td>
                    <td>{todayKcal} Kcal</td>
                    <td>{thisWeekKcal} Kcal</td>
                    <td>{weekKcal} Kcal</td>
                    <td>{monthKcal} Kcal</td>
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
                      <td>{TimeUtils.formatTimeDate(session!.timestamp)}</td>
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
                      <td>{session!.active_calories}</td>
                      <td>{session!.training_load}</td>
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
                          {TimeUtils.formatTimeDate(workout.latest_session)}
                        </td>
                        <td>{workout.sessions}</td>
                        <td>{TimeUtils.formatDuration(workout.avg_time)}</td>
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
          />
        )}
      </div>

      <ImportSessionsMenu
        availableDevices={availableDevices}
        onImported={refresh}
      />
    </>
  );
}
