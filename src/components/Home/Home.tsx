import { AppContext } from "@/context/AppContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { LoadingContext } from "@/context/LoadingContext";
import { BackendClient } from "@/utils/backend/client";
import { BackendListener } from "@/utils/backend/listener";
import { SessionUtils, WorkoutLoad } from "@/utils/SessionUtils";
import { Button, Menu, MenuItem } from "@mui/material";
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

export function Home() {
  const { availableDevices } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);

  const [workload, setWorkload] = useState<WorkoutLoad[]>([]);
  const [minDate, setMinDate] = useState(0);
  const [importMenuAnchor, setImportMenuAnchor] = useState<{
    top: number;
    left: number;
  } | null>(null);

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
    BackendClient.getSessions()
      .then((data) => {
        const workout_data = SessionUtils.calculateWorkoutLoad(data);
        setWorkload(workout_data);
        console.table(workout_data);
        if (workout_data.length > 0) {
          setMinDate(workout_data[0].date);
        }
      })
      .finally(() => {
        finishLoading();
      });
  };

  useEffect(() => {
    const unregisterSessionAdded = BackendListener.onSessionsAdded(() => {
      refresh();
    });

    refresh();

    return () => {
      unregisterSessionAdded();
    };
  }, []);

  return (
    <>
      {workload.length > 0 && (
        <div className="chart-container">
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
