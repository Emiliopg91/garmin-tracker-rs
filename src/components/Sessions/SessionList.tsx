import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionListItem } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { Button, Menu, MenuItem } from "@mui/material";
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
import { BackendListener } from "@/utils/backend/listener";
import { TimeUtils } from "@/utils/TimeUtils";
import {
  SessionFrontDetails,
  SessionUtils,
  WorkoutLoad,
} from "@/utils/SessionUtils";

export function SessionsList() {
  const { startLoading, finishLoading, availableDevices, translate, settings } =
    useContext(AppContext);

  const [minDate, setMinDate] = useState(0);
  const [workload, setWorkload] = useState<WorkoutLoad[]>([]);
  const [sessions, setSessions] = useState<SessionListItem[]>([]);
  const [sessionDetails, setSessionDetails] = useState<
    SessionFrontDetails | undefined
  >(undefined);
  const [importMenuAnchor, setImportMenuAnchor] = useState<HTMLElement | null>(
    null,
  );

  const refreshList = () => {
    startLoading();
    BackendClient.getSessions()
      .then((data) => {
        setSessions(data);
        const workout_data = SessionUtils.calculateWorkoutLoad(data);
        setWorkload(workout_data);
        if (workout_data.length > 0) {
          setMinDate(workout_data[0].date);
        }
      })
      .finally(() => {
        finishLoading();
      });
  };

  useEffect(() => {
    refreshList();
    const unregisterSessionAdded = BackendListener.onSessionsAdded(() => {
      refreshList();
    });

    const unregisterSessionLocation = BackendListener.onSessionLocationUpdate(
      (data) => {
        setSessions((prev) => {
          return prev.map((item) => {
            if (item.timestamp !== data.session) return item;
            return { ...item, name: data.location };
          });
        });
      },
    );

    return () => {
      unregisterSessionAdded();
      unregisterSessionLocation();
    };
  }, []);

  const importDevice = (serial: string) => {
    startLoading();
    BackendClient.importFromDevice(serial)
      .then((count) => {
        if (count > 0) {
          refreshList();
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
              <th style={{ textAlign: "center" }}>{translate("date")}</th>
              <th style={{ textAlign: "center" }}>{translate("sport")}</th>
              <th style={{ textAlign: "center" }}>{translate("name")}</th>
              <th style={{ textAlign: "center" }}>
                {translate("active_calories")}
              </th>
              <th style={{ textAlign: "center" }}>
                {translate("workout_load")}
              </th>
            </tr>
          </thead>

          <tbody>
            {sessions.map((session, idx) => (
              <tr
                key={idx}
                onClick={() => getSessionDetails(session.timestamp)}
                style={{ cursor: "pointer" }}
              >
                <td>{TimeUtils.formatTimeDate(session.timestamp)}</td>
                <td>{session.sport}</td>
                <td>{session.name}</td>
                <td>{session.active_calories}</td>
                <td>{session.training_load}</td>
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
      {availableDevices.length > 0 && (
        <div style={{ padding: "5px", width: "100%", marginTop: "auto" }}>
          {availableDevices.length == 1 && (
            <Button
              id="import-file-button"
              variant="contained"
              style={{ width: "100%" }}
              onClick={() => {
                importDevice(availableDevices[0].serial_number);
              }}
            >
              {translate("import_sessions_from_device", [
                availableDevices[0].manufacturer +
                  " " +
                  availableDevices[0].model,
              ])}
            </Button>
          )}
          {availableDevices.length > 1 && (
            <>
              <Button
                id="import-file-toggle"
                variant="contained"
                style={{ width: "100%" }}
                onClick={(e) => setImportMenuAnchor(e.currentTarget)}
              >
                {translate("import_sessions")}
              </Button>
              <Menu
                id="import-file-menu"
                anchorEl={importMenuAnchor}
                open={Boolean(importMenuAnchor)}
                onClose={() => setImportMenuAnchor(null)}
              >
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
      )}
    </>
  );
}
