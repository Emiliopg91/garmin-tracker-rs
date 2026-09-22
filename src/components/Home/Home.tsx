import { AppContext } from "@/context/AppContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { LoadingContext } from "@/context/LoadingContext";
import { BackendClient } from "@/utils/backend/client";
import {
  SessionFrontDetails,
  SessionUtils,
  WorkoutLoad,
} from "@/utils/SessionUtils";
import { Button, Menu, MenuItem } from "@mui/material";
import { usePickerAdapter } from "@mui/x-date-pickers/hooks";
import { useContext, useEffect, useState } from "react";
import {
  Area,
  CartesianGrid,
  ComposedChart,
  Legend,
  Line,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";
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

export function Home() {
  const { availableDevices, sessionsVersion } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);
  const adapter = usePickerAdapter();

  const [day, setDay] = useState(() => new Date().toDateString());
  const [availableData, setAvailableData] = useState(false);
  const [workload, setWorkload] = useState<WorkoutLoad[]>([]);
  const [minDate, setMinDate] = useState(0);
  const [importMenuAnchor, setImportMenuAnchor] = useState<{
    top: number;
    left: number;
  } | null>(null);

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

  const importDevice = (serial: string) => {
    startLoading();
    BackendClient.importFromDevice(serial)
      .then((count) => {
        if (count > 0) {
          refresh();
        }
      })
      .finally(() => {
        finishLoading();
      });
  };

  const importFromDisk = () => {
    startLoading();
    BackendClient.importFromFiles()
      .then((count) => {
        if (count > 0) {
          refresh();
        }
      })
      .finally(() => {
        finishLoading();
      });
  };
  const refresh = () => {
    startLoading();
    const today = new Date(new Date().setHours(0, 0, 0, 0)).getTime() / 1000;
    BackendClient.getSessions(
      today - SessionUtils.CHRONIC_DAYS * 2 * 24 * 60 * 60,
    )
      .then((data) => {
        setAvailableData(data.length > 0);

        if (data.length > 0) {
          const workout_data = SessionUtils.calculateWorkoutLoad(data);
          const overreaching = SessionUtils.isOverreaching(workout_data);
          setRest(overreaching);

          startLoading();
          BackendClient.getWorkoutList()
            .then((data) => {
              const filtered = data.filter((w) => w.enabled);
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

          for (let i = 0; i < data.length; i++) {
            if (data[i].timestamp < monthLimit) {
              break;
            }
            monthTmp += data[i].total_elapsed_time;
            monthKcl += data[i].active_calories;
            monthLoad += data[i].training_load;
            if (data[i].timestamp >= weekLimit) {
              weekTmp += data[i].total_elapsed_time;
              weekKcl += data[i].active_calories;
              weekLoad += data[i].training_load;

              if (data[i].timestamp >= thisWeekLimit) {
                thisWeekTmp += data[i].total_elapsed_time;
                thisWeekKcl += data[i].active_calories;
                thisWeekLoad += data[i].training_load;

                if (data[i].timestamp >= today) {
                  todayTmp += data[i].total_elapsed_time;
                  todayKcl += data[i].active_calories;
                  todayLoad += data[i].training_load;
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

          setLastSession(data[0]);
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
              {workload.length > 0 && (
                <div className="chart-container ">
                  <ResponsiveContainer
                    className="chart-responsive"
                    width="100%"
                    height="100%"
                  >
                    <ComposedChart data={workload}>
                      <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
                      <XAxis
                        dataKey="date"
                        type="number"
                        domain={[minDate, workload[workload.length - 1].date]}
                        stroke="#fff"
                        tick={false}
                        height={0}
                      />
                      <YAxis
                        yAxisId="left"
                        stroke="#fff"
                        width={0}
                        domain={[0, 1]}
                        tick={false}
                      />{" "}
                      <Area
                        dataKey="lower"
                        stackId="1"
                        stroke="none"
                        type="monotone"
                        legendType="none"
                        fill="transparent"
                        dot={false}
                        isAnimationActive={false}
                        activeDot={false}
                      />
                      <Area
                        dataKey="upper"
                        stackId="1"
                        stroke="none"
                        type="monotone"
                        fill="lightgreen"
                        legendType="none"
                        fillOpacity={0.1}
                        dot={false}
                        isAnimationActive={false}
                        activeDot={false}
                      />
                      <Line
                        type="monotone"
                        name={translate("workload")}
                        dataKey="current"
                        stroke="green"
                        dot={{ fill: "green" }}
                        isAnimationActive={false}
                        activeDot={false}
                      />
                      <Line
                        type="monotone"
                        name={translate("reference")}
                        legendType="line"
                        dataKey="reference"
                        stroke="#ffffff40"
                        dot={false}
                        isAnimationActive={false}
                        activeDot={false}
                      />
                      <Legend />
                    </ComposedChart>
                  </ResponsiveContainer>
                </div>
              )}
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
              </div>{" "}
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

      <div className="list-action-bar">
        {availableDevices.length == 0 && (
          <Button
            id="import-file-toggle"
            variant="contained"
            className="full-width-button"
            onClick={importFromDisk}
          >
            {translate("import_from_disk")}
          </Button>
        )}
        {availableDevices.length > 0 && (
          <>
            <Button
              id="import-file-toggle"
              variant="contained"
              className="full-width-button"
              onClick={(e) =>
                setImportMenuAnchor({ top: e.clientY, left: e.clientX })
              }
            >
              {translate("import_sessions")}
            </Button>
            <Menu
              id="import-file-menu"
              anchorReference="anchorPosition"
              anchorPosition={importMenuAnchor ?? undefined}
              open={Boolean(importMenuAnchor)}
              onClose={() => setImportMenuAnchor(null)}
              anchorOrigin={{ vertical: "top", horizontal: "left" }}
              transformOrigin={{ vertical: "bottom", horizontal: "left" }}
            >
              <MenuItem
                onClick={() => {
                  setImportMenuAnchor(null);
                  importFromDisk();
                }}
              >
                {translate("import_from_disk")}
              </MenuItem>
              {availableDevices.map((device, idx) => (
                <MenuItem
                  key={"dev-" + idx}
                  onClick={() => {
                    setImportMenuAnchor(null);
                    importDevice(device.serial_number);
                  }}
                >
                  {translate("import_from_device", [
                    device.manufacturer + " " + device.model,
                  ])}
                </MenuItem>
              ))}
            </Menu>
          </>
        )}
      </div>
    </>
  );
}
