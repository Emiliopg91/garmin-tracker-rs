import { AppContext } from "@/context/AppContext";
import { WorkoutDetails } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";
import { useContext, useEffect, useState } from "react";
import { Dialog, DialogContent, DialogTitle, IconButton } from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";

type Props = {
  workout: WorkoutDetails;
  onClose: () => void;
};

export function WorkoutModal({ workout, onClose }: Props) {
  const { translate, settings } = useContext(AppContext);
  const [chartData, setChartData] = useState<
    { date: number; volume: number }[]
  >([]);
  const [minVol, setMinVol] = useState(99999);
  const [maxVol, setMaxVol] = useState(0);
  const [minDate, setMinDate] = useState(99999);
  const [maxDate, setMaxDate] = useState(0);

  useEffect(() => {
    const data = [...workout.sessions].reverse().map((ws) => {
      const [dd, mm, yyyy] = TimeUtils.formatTimeDate(ws.date)
        .split(" ")[1]
        .split("/")
        .map(Number);
      const date = new Date(yyyy, mm - 1, dd);
      return {
        date: date.getTime(),
        volume: ws.volume,
      };
    });
    setChartData(data);
    const dates = [...data].map(({ date }) => {
      return date;
    });
    setMinDate(Math.min(...dates));
    setMaxDate(Math.max(...dates));
    const volumes = [...data].map(({ volume }) => {
      return volume;
    });
    setMinVol(Math.min(...volumes));
    setMaxVol(Math.max(...volumes));
  }, []);

  return (
    <Dialog open={true} onClose={onClose} fullWidth maxWidth="md">
      <DialogTitle>
        {workout.name.length > 0 && <span>{workout.name}</span>}
        {workout.name.length == 0 && <span>{translate("other")}</span>}
        <IconButton onClick={onClose} className="modal-close-button">
          <CloseIcon />
        </IconButton>
      </DialogTitle>

      <DialogContent dividers>
        <table id="workout-details-table">
          <colgroup>
            <col className="col-200" />
            <col className="col-150" />
            <col />
          </colgroup>
          <tbody>
            <tr>
              <td>{translate("sessions")}:</td>
              <td>{workout.session_count}</td>
            </tr>
            <tr>
              <td>{translate("latest_session")}</td>
              <td>{TimeUtils.formatTimeDate(workout.latest_session)}</td>
            </tr>
            <tr>
              <td>{translate("average_time")}</td>
              <td>{TimeUtils.formatDuration(workout.avg_time)}</td>
            </tr>
            {workout.name.length > 0 && (
              <tr>
                <td>{translate("average_volume")}:</td>
                <td>
                  {UnitUtils.fromKg(
                    workout.avg_volume,
                    settings.weight_unit,
                  ).toFixed(1)}{" "}
                  {UnitUtils.getUnit(settings.weight_unit)}
                </td>
              </tr>
            )}
          </tbody>
        </table>
        {workout.sessions.length > 1 && (
          <>
            <hr />
            {workout.avg_volume > 0 && (
              <>
                <div className="chart-container">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart
                      data={chartData}
                      margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
                    >
                      <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
                      <XAxis
                        dataKey="date"
                        stroke="#fff"
                        type="number"
                        domain={[minDate, maxDate]}
                        tick={false}
                        height={0}
                      />
                      <YAxis
                        stroke="#fff"
                        width={0}
                        domain={[minVol * 0.9, maxVol * 1.1]}
                        tick={false}
                      />{" "}
                      {/* ← número, no "auto" */}
                      <Line
                        name={translate("volume")}
                        type="monotone"
                        dataKey="volume"
                        stroke="#0f0"
                        dot={{ fill: "#0f0" }}
                        activeDot={{ stroke: "#00ff0000" }}
                        isAnimationActive={false}
                      />
                      <ReferenceLine
                        y={(minVol + maxVol) / 2}
                        stroke="#808080"
                        strokeDasharray="10 5"
                      />
                      <Legend />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </>
            )}
          </>
        )}
        <hr />
        <table>
          <colgroup>
            <col className={workout.avg_volume > 0 ? "col-230" : "col-370"} />
            <col className={workout.avg_volume > 0 ? "col-120" : "col-260"} />
            {workout.avg_volume > 0 && <col className="col-280" />}
          </colgroup>
          <thead>
            <tr>
              <th>{translate("date")}</th>
              <th>{translate("time")}</th>
              {workout.avg_volume > 0 && <th>{translate("volume")}</th>}
            </tr>
          </thead>
          <tbody>
            {workout.sessions.map((session, idx) => (
              <tr key={idx} className="divider-bottom">
                <td className="text-center">
                  {TimeUtils.formatTimeDate(session.date)}
                </td>
                <td className="text-center">
                  {TimeUtils.formatDuration(session.time)}
                </td>
                {workout.avg_volume > 0 && (
                  <td className="text-center">
                    {UnitUtils.fromKg(
                      session.volume,
                      settings.weight_unit,
                    ).toFixed(1)}{" "}
                    {UnitUtils.getUnit(settings.weight_unit)}
                    {idx < workout.sessions.length - 1 &&
                      " (" +
                        new Intl.NumberFormat("es-ES", {
                          style: "percent",
                          minimumFractionDigits: 2,
                          maximumFractionDigits: 2,
                          signDisplay: "always",
                        }).format(
                          (workout.sessions[idx].volume -
                            workout.sessions[idx + 1].volume) /
                            workout.sessions[idx + 1].volume,
                        ) +
                        ")"}
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </DialogContent>
    </Dialog>
  );
}
