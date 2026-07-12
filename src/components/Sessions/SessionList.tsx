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
  const { setLoading, availableDevices, translate } = useContext(AppContext);

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
          const startOfDay = (d: Date) =>
            new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();

          const addDays = (ts: number, n: number) => {
            const d = new Date(ts);
            d.setDate(d.getDate() + n);
            return d.getTime();
          };

          const CHRONIC_DAYS = 28;
          const ACUTE_DAYS = 7;
          const ACWR_UPPER_RATIO = 1;
          const ACWR_LOWER_RATIO = 0.7;
          const TODAY = startOfDay(new Date());

          const LAMBDA_ACUTE = 2 / (ACUTE_DAYS + 1); // ~0.25
          const LAMBDA_CHRONIC = 2 / (CHRONIC_DAYS + 1); // ~0.069

          let working_data = data
            .map((s) => {
              const [dd, mm, yyyy] = s.date
                .split(" ")[1]
                .split("/")
                .map(Number);
              const date = new Date(yyyy, mm - 1, dd).getTime();

              return { date: date, volume: s.volume };
            })
            .filter(
              (s) => TODAY - 2 * CHRONIC_DAYS * 24 * 60 * 60 * 1000 <= s.date,
            );

          for (
            let dat = addDays(TODAY, -2 * CHRONIC_DAYS + 1);
            dat <= TODAY;
            dat = addDays(dat, 1)
          ) {
            if (!working_data.find(({ date }) => date === dat)) {
              working_data.push({ date: dat, volume: 0 });
            }
          }

          working_data = working_data.sort((a, b) => a.date - b.date);

          if (working_data.length === 0) {
            setWorkload([]);
          } else {
            let ewmaAcute = working_data[0].volume;
            let ewmaChronic = working_data[0].volume;

            const ewmaSeries: {
              date: number;
              acute: number;
              chronic: number;
            }[] = [
              {
                date: working_data[0].date,
                acute: ewmaAcute,
                chronic: ewmaChronic,
              },
            ];

            for (let i = 1; i < working_data.length; i++) {
              const v = working_data[i].volume;
              ewmaAcute = v * LAMBDA_ACUTE + ewmaAcute * (1 - LAMBDA_ACUTE);
              ewmaChronic =
                v * LAMBDA_CHRONIC + ewmaChronic * (1 - LAMBDA_CHRONIC);

              ewmaSeries.push({
                date: working_data[i].date,
                acute: ewmaAcute,
                chronic: ewmaChronic,
              });
            }

            let load_data = ewmaSeries
              .filter((_, idx) => idx >= CHRONIC_DAYS)
              .map((e) => ({
                date: e.date,
                upper: e.chronic * ACWR_UPPER_RATIO,
                lower: e.chronic * ACWR_LOWER_RATIO,
                current: e.acute,
                reference: e.chronic,
              }));

            if (load_data.length === 0) {
              setWorkload([]);
            } else {
              const max_load = load_data.reduce(
                (max, e) => Math.max(max, e.upper),
                0,
              );
              const safeMaxLoad = max_load > 0 ? max_load : 1;

              load_data = load_data.map((e) => ({
                ...e,
                current: e.current / safeMaxLoad,
                upper: e.upper / safeMaxLoad,
                lower: e.lower / safeMaxLoad,
                reference: e.reference / safeMaxLoad,
              }));

              setMinDate(load_data[0].date);
              setWorkload(load_data);
            }
          }
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

        <table>
          <thead>
            <tr>
              <th style={{ textAlign: "center" }}>{translate("workout")}</th>
              <th style={{ textAlign: "center" }}>{translate("date")}</th>
              <th style={{ textAlign: "center" }}>{translate("exercises")}</th>
              <th style={{ textAlign: "center" }}>{translate("series")}</th>
              <th style={{ textAlign: "center" }}>{translate("volume")}</th>
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
            {translate("import_sessions")}
          </Dropdown.Toggle>

          <Dropdown.Menu id="import-file-menu">
            <Dropdown.Item key={"file"} onClick={importFile}>
              {translate("import_from_file")}
            </Dropdown.Item>
            {availableDevices.length > 0 &&
              availableDevices.map((device, idx) => (
                <Dropdown.Item
                  key={"dev-" + idx}
                  onClick={() => {
                    importDevice(device.serial_number);
                  }}
                >
                  {translate("import_from_device", [
                    device.manufacturer + " " + device.model,
                  ])}
                </Dropdown.Item>
              ))}
            {availableDevices.length == 0 && (
              <Dropdown.Item disabled={true}>
                {translate("no_device_found")}
              </Dropdown.Item>
            )}
          </Dropdown.Menu>
        </Dropdown>
      </div>
    </>
  );
}
