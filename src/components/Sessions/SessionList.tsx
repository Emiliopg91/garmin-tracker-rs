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
  reference: number;
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

        if (data.length === 0) {
          setWorkload([]);
        } else {
          const ONE_DAY_MS = 24 * 60 * 60 * 1000;
          const DAYS = 28;
          const AVG_HR_WEIGHT = 0.7;
          const MAX_HR_WEIGHT = 0.3;
          const HR_FACTOR_MIN = 0.6;
          const HR_FACTOR_MAX = 1.2;
          const ACWR_UPPER_RATIO = 1;
          const ACWR_LOWER_RATIO = 0.7;

          const clamp = (v: number, min: number, max: number) =>
            Math.max(min, Math.min(v, max));
          const startOfDay = (d: Date) =>
            new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();

          const now = startOfDay(new Date());

          let estimatedMaxHr = 180;
          for (const s of data) {
            if (s.max_heart_rate > estimatedMaxHr)
              estimatedMaxHr = s.max_heart_rate;
          }

          const calculateLoad = (
            volume: number,
            avgHr: number,
            maxHr: number,
          ) => {
            const hrFactor = clamp(
              AVG_HR_WEIGHT * (avgHr / estimatedMaxHr) +
                MAX_HR_WEIGHT * (maxHr / estimatedMaxHr),
              HR_FACTOR_MIN,
              HR_FACTOR_MAX,
            );
            return volume * hrFactor;
          };

          const dailyMap = new Map<number, number>();
          for (const s of data) {
            const [dd, mm, yyyy] = s.date.split(" ")[1].split("/").map(Number);
            const date = new Date(yyyy, mm - 1, dd).getTime();
            dailyMap.set(
              date,
              (dailyMap.get(date) ?? 0) +
                calculateLoad(s.volume, s.avg_heart_rate, s.max_heart_rate),
            );
          }

          const lambda7 = 2 / 8;
          const lambda28 = 2 / 29;

          let acute = 0;
          let chronic = 0;
          let max = 0;

          const workoutData: WorkoutLoad[] = new Array(DAYS);

          for (let i = 0; i < DAYS; i++) {
            const date = now - (DAYS - 1 - i) * ONE_DAY_MS;
            const load = dailyMap.get(date) ?? 0;

            if (i === 0) {
              acute = load;
              chronic = load;
            } else {
              acute = lambda7 * load + (1 - lambda7) * acute;
              chronic = lambda28 * load + (1 - lambda28) * chronic;
            }

            const upper = chronic * ACWR_UPPER_RATIO;
            const lower = chronic * ACWR_LOWER_RATIO;
            if (upper > max) max = upper;

            workoutData[i] = {
              date,
              current: acute,
              reference: chronic,
              upper,
              lower,
            };
          }

          const normalized = workoutData.map((d) => ({
            date: d.date,
            current: d.current / max,
            reference: d.reference / max,
            upper: d.upper / max,
            lower: d.lower / max,
          }));

          setMinDate(normalized[0].date);
          setWorkload(normalized);
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
                  name="Workload"
                  dataKey="current"
                  stroke="green"
                  dot={{ fill: "green" }}
                  isAnimationActive={false}
                  activeDot={false}
                />
                <Line
                  type="monotone"
                  name="Reference"
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
