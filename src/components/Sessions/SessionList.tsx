import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionDetails, SessionListItem } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { Dropdown } from "react-bootstrap";
import { SessionModal } from "./SessionModal";
import {
  Area,
  CartesianGrid,
  Legend,
  Line,
  ComposedChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";

type WorkoutLoad = {
  date: number;
  upper: number;
  current: number;
  lower: number;
};

export function SessionsList() {
  const { setLoading, availableDevices } = useContext(AppContext);

  const [minDate, setMinDate] = useState(0);
  const [workload, setWorkload] = useState<WorkoutLoad[]>([]);
  const [sessions, setSessions] = useState<SessionListItem[]>([]);
  const [sessionDetails, setSessionDetails] = useState<
    SessionDetails | undefined
  >(undefined);

  const refreshList = () => {
    setLoading(true);
    BackendClient.getSessions()
      .then((data) => {
        setSessions(data);

        if (data.length == 0) {
          setWorkload([]);
        } else {
          let now = new Date();
          now = new Date(now.getFullYear(), now.getMonth(), now.getDate());

          const ONE_DAY_MS = 24 * 60 * 60 * 1000;
          const SEVEN_DAYS_MS = 7 * ONE_DAY_MS;
          const FOUR_WEEKS_MS = 4 * SEVEN_DAYS_MS;
          let max_ot = 0;

          const mod_data = data.map((s) => {
            const [dd, mm, yyyy] = s.date.split(" ")[1].split("/").map(Number);
            const cur_date = new Date(yyyy, mm - 1, dd);
            return { date: cur_date.getTime(), volume: s.volume };
          });

          for (let i = 0; i < 28; i++) {
            const f_day = now.getTime() - i * ONE_DAY_MS;
            if (mod_data.filter(({ date }) => f_day == date).length == 0) {
              mod_data.push({ date: f_day, volume: 0 });
            }
          }

          let workoutData: WorkoutLoad[] = [];
          mod_data.map((session) => {
            const filtered_7 = mod_data.filter((s) => {
              return (
                session.date >= s.date && session.date - s.date < SEVEN_DAYS_MS
              );
            });
            const avg_7 =
              filtered_7.map((s) => s.volume).reduce((acc, n) => acc + n, 0) /
              filtered_7.length;

            const filtered_28 = mod_data.filter((s) => {
              return (
                session.date >= s.date && session.date - s.date < FOUR_WEEKS_MS
              );
            });
            const avg_28 =
              filtered_28.map((s) => s.volume).reduce((acc, n) => acc + n, 0) /
              filtered_28.length;

            const ot = avg_28 * 1.5;
            const ut = avg_28 * 0.8;

            if (ot > max_ot) {
              max_ot = ot;
            }
            workoutData.push({
              date: session.date,
              upper: ot,
              current: avg_7,
              lower: ut,
            });
          });
          workoutData = workoutData
            .filter((wl) => {
              return now.getTime() - wl.date < FOUR_WEEKS_MS;
            })
            .map((wl) => ({
              date: wl.date,
              current: (wl.current ? wl.current : 0) / max_ot,
              upper: wl.upper / max_ot,
              lower: wl.lower / max_ot,
            }));

          workoutData = workoutData.filter(
            (s) => s.date >= now.getTime() - FOUR_WEEKS_MS,
          );
          workoutData = workoutData.sort((a, b) => a.date - b.date);

          setMinDate(Math.min(...workoutData.map((s) => s.date)));
          setWorkload(workoutData);
        }
      })
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    refreshList();
  }, []);

  const importFile = () => {
    setLoading(true);
    BackendClient.importFromFile()
      .then((count) => {
        if (count > 0) {
          refreshList();
        }
      })
      .finally(() => {
        setLoading(false);
      });
  };

  const importDevice = (serial: string) => {
    setLoading(true);
    BackendClient.importFromDevice(serial)
      .then((count) => {
        if (count > 0) {
          refreshList();
        }
      })
      .finally(() => {
        setLoading(false);
      });
  };

  const getSessionDetails = (timestamp: string) => {
    setLoading(true);
    BackendClient.getSessionDetails(timestamp)
      .then((details) => {
        setSessionDetails(details);
      })
      .finally(() => {
        setLoading(false);
      });
  };

  return (
    <>
      <div id="list-layer">
        {workload.length > 0 && (
          <div style={{ width: "100%", height: 200 }}>
            <ResponsiveContainer width="100%" height="100%">
              <ComposedChart
                data={workload}
                margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
              >
                <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
                <XAxis
                  dataKey="date"
                  type="number"
                  domain={[minDate, new Date().getTime()]}
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
                  legendType="none"
                  fill="transparent"
                  dot={false}
                  isAnimationActive={false}
                  activeDot={false}
                />
                <Area
                  dataKey="upper"
                  legendType="none"
                  stackId="1"
                  stroke="none"
                  fill="lightgreen"
                  fillOpacity={0.2}
                  dot={false}
                  isAnimationActive={false}
                  activeDot={false}
                />
                <Line
                  yAxisId="left"
                  type="monotone"
                  name="Workload"
                  dataKey="current"
                  stroke="cyan"
                  dot={{ fill: "cyan" }}
                  isAnimationActive={false}
                  activeDot={false}
                />
                <Legend />
              </ComposedChart>
            </ResponsiveContainer>
          </div>
        )}

        <table>
          <thead>
            <tr>
              <th style={{ textAlign: "center" }}>Workout</th>
              <th style={{ textAlign: "center" }}>Date</th>
              <th style={{ textAlign: "center" }}>Exercises</th>
              <th style={{ textAlign: "center" }}>Sets</th>
              <th style={{ textAlign: "center" }}>Volume</th>
            </tr>
          </thead>

          <tbody>
            {sessions.map((session, idx) => (
              <tr
                key={idx}
                onClick={() => getSessionDetails(session.timestamp)}
                style={{ cursor: "pointer" }}
              >
                <td>{session.name}</td>
                <td>{session.date}</td>
                <td>{session.exercises_num}</td>
                <td>{session.series_num}</td>
                <td>{session.volume} Kg</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div>
        {sessionDetails && (
          <SessionModal
            session={sessionDetails}
            onClose={() => setSessionDetails(undefined)}
            onUpdate={() => refreshList()}
          />
        )}
      </div>
      <div style={{ padding: "5px", width: "100%", marginTop: "auto" }}>
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
    </>
  );
}
