import { AppContext } from "@/context/AppContext";
import { WorkoutDetails } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { Modal } from "react-bootstrap";
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
  const { translate } = useContext(AppContext);
  const [chartData, setChartData] = useState<
    { date: number; volume: number }[]
  >([]);
  const [minVol, setMinVol] = useState(99999);
  const [maxVol, setMaxVol] = useState(0);
  const [minDate, setMinDate] = useState(99999);
  const [maxDate, setMaxDate] = useState(0);

  useEffect(() => {
    const data = [...workout.sessions].reverse().map((ws) => {
      const [dd, mm, yyyy] = ws.date.split(" ")[1].split("/").map(Number);
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
    <div
      className="modal show"
      style={{ display: "block", position: "initial" }}
    >
      <Modal show={true} onHide={onClose} data-bs-theme="dark">
        <Modal.Header closeButton>
          <Modal.Title>
            {workout.name.length > 0 && <span>{workout.name}</span>}
            {workout.name.length == 0 && <span>{translate("other")}</span>}
          </Modal.Title>
        </Modal.Header>

        <Modal.Body>
          <table id="workout-details-table">
            <colgroup>
              <col style={{ width: "200px" }} />
              <col style={{ width: "150px" }} />
              <col />
            </colgroup>
            <tbody>
              <tr>
                <td>{translate("sessions")}:</td>
                <td>{workout.session_count}</td>
              </tr>
              <tr>
                <td>{translate("latest_session")}</td>
                <td>{workout.latest_session}</td>
              </tr>
              <tr>
                <td>{translate("average_time")}</td>
                <td>{workout.avg_time}</td>
              </tr>
              {workout.name.length > 0 && (
                <tr>
                  <td>{translate("average_volume")}:</td>
                  <td>{Math.floor(workout.avg_volume)} Kg</td>
                </tr>
              )}
            </tbody>
          </table>
          {workout.sessions.length > 0 && (
            <>
              <hr />
              {workout.name.length > 0 && (
                <>
                  <div style={{ width: "100%", height: 200 }}>
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart
                        data={chartData}
                        margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
                      >
                        <CartesianGrid
                          stroke="#80808000"
                          strokeDasharray="5 5"
                        />
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
                  <br />
                  <hr />
                </>
              )}
              <h5 style={{ textAlign: "center" }}>{translate("sessions")}</h5>
              <table>
                <colgroup>
                  <col
                    style={{
                      width: workout.name.length > 0 ? "230px" : "370px",
                    }}
                  />
                  <col
                    style={{
                      width: workout.name.length > 0 ? "120px" : "260px",
                    }}
                  />
                  {workout.name.length > 0 && (
                    <col style={{ width: "280px" }} />
                  )}
                </colgroup>
                <thead>
                  <tr>
                    <th>{translate("date")}</th>
                    <th>{translate("time")}</th>
                    {workout.name.length > 0 && <th>{translate("volume")}</th>}
                  </tr>
                </thead>
                <tbody>
                  {workout.sessions.map((session, idx) => (
                    <tr
                      key={idx}
                      style={{
                        borderBottom: "1px solid #e4e4e430",
                      }}
                    >
                      <td>{session.date}</td>
                      <td>{session.time}</td>
                      {workout.name.length > 0 && (
                        <td>
                          {session.volume} Kg{" "}
                          {session.vol_diff != "-" &&
                            "(" + session.vol_diff + ")"}
                        </td>
                      )}
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </Modal.Body>
      </Modal>
    </div>
  );
}
