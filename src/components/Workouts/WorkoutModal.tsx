import { WorkoutDetails } from "@/utils/backend/models";
import { useEffect, useState } from "react";
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
  const [chartData, setChartData] = useState<
    { date: string; volume: number }[]
  >([]);
  const [minVol, setMinVol] = useState(99999);
  const [maxVol, setMaxVol] = useState(0);

  useEffect(() => {
    const data = [...workout.sessions].reverse().map((ws) => {
      return {
        date: ws.date.split(" ")[1],
        volume: ws.volume,
      };
    });
    setChartData(data);
    const volumes = [...data].map(({ volume }) => {
      return volume;
    });
    setMinVol(Math.min(...volumes));
    setMaxVol(Math.max(...volumes));
    console.table(data);
  }, []);

  return (
    <div
      className="modal show"
      style={{ display: "block", position: "initial" }}
    >
      <Modal show={true} onHide={onClose} data-bs-theme="dark">
        <Modal.Header closeButton>
          <Modal.Title>{workout.name}</Modal.Title>
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
                <td>Sessions:</td>
                <td>{workout.session_count}</td>
              </tr>
              <tr>
                <td>Latest session:</td>
                <td>{workout.latest_session}</td>
              </tr>
              <tr>
                <td>Average time:</td>
                <td>{workout.avg_time}</td>
              </tr>
              <tr>
                <td>Average volume:</td>
                <td>{Math.floor(workout.avg_volume)} Kg</td>
              </tr>
            </tbody>
          </table>
          {workout.sessions.length > 0 && (
            <>
              <hr />
              <div style={{ width: "100%", height: 200 }}>
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart
                    data={chartData}
                    margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
                  >
                    <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
                    <XAxis
                      dataKey="date"
                      stroke="#fff"
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
                      name="Volume"
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
              <h5 style={{ textAlign: "center" }}>Sessions</h5>
              <table>
                <colgroup>
                  <col style={{ width: "230px" }} />
                  <col style={{ width: "120px" }} />
                  <col style={{ width: "280px" }} />
                </colgroup>
                <thead>
                  <tr>
                    <th>Date</th>
                    <th>Time</th>
                    <th>Volume</th>
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
                      <td>
                        {session.volume} Kg{" "}
                        {session.vol_diff != "-" &&
                          "(" + session.vol_diff + ")"}
                      </td>
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
