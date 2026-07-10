import { AppContext } from "@/context/AppContext";
import { ExerciseDetails } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { Modal } from "react-bootstrap";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";

type Props = {
  exercise: ExerciseDetails;
  onClose: () => void;
};

export function ExerciseModal({ exercise, onClose }: Props) {
  const { translate } = useContext(AppContext);
  const [chartData, setChartData] = useState<
    { date: number; volume: number; reps: number }[]
  >([]);
  const [maxVol, setMaxVol] = useState(0);
  const [minDate, setMinDate] = useState(99999);
  const [maxDate, setMaxDate] = useState(0);

  useEffect(() => {
    const data: { date: number; volume: number; reps: number }[] = [];
    Object.keys(exercise.series).forEach((k) => {
      const [dd, mm, yyyy] = k
        .split("\n")[1]
        .split(" ")[1]
        .split("/")
        .map(Number);
      const date = new Date(yyyy, mm - 1, dd);

      let count = 0;
      let weight = 0;
      exercise.series[k].forEach((s) => {
        count += s.reps;
        weight += s.reps * s.weight;
      });
      data.push({
        date: date.getTime(),
        volume: weight,
        reps: count,
      });
    });
    data.sort((a, b) => a.date - b.date);
    setChartData(data);
    const dates = [...data].map(({ date }) => {
      return date;
    });
    setMinDate(Math.min(...dates));
    setMaxDate(Math.max(...dates));
    const volumes = data.map(({ volume }) => {
      return volume;
    });
    setMaxVol(Math.max(...volumes));
  }, []);

  return (
    <div
      className="modal show"
      style={{ display: "block", position: "initial" }}
    >
      <Modal show={true} onHide={onClose} data-bs-theme="dark">
        <Modal.Header closeButton>
          <Modal.Title>{exercise.name}</Modal.Title>
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
                <td>{translate("personal_record")}:</td>
                <td>{exercise.reps + "x" + exercise.weight + " Kg"}</td>
              </tr>
              <tr>
                <td>{translate("record_date")}:</td>
                <td>{exercise.pr_date}</td>
              </tr>
              <tr>
                <td>{translate("rm")}:</td>
                <td>{Math.floor(exercise.rm!) + " Kg"}</td>
              </tr>
            </tbody>
          </table>
          {Object.keys(exercise.workouts).length > 0 && (
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
                      type="number"
                      domain={[minDate, maxDate]}
                      stroke="#fff"
                      tick={false}
                      height={0}
                    />
                    <YAxis
                      yAxisId="left"
                      stroke="#fff"
                      width={0}
                      domain={[0, maxVol]}
                      tick={false}
                    />{" "}
                    <YAxis
                      yAxisId="right"
                      stroke="#fff"
                      width={0}
                      tick={false}
                    />{" "}
                    <Line
                      yAxisId="left"
                      type="monotone"
                      name={translate("volume")}
                      dataKey="volume"
                      stroke="#0f0"
                      dot={{ fill: "#0f0" }}
                      activeDot={{ stroke: "#f0f0f000" }}
                      isAnimationActive={false}
                    />
                    <Line
                      yAxisId="right"
                      type="monotone"
                      name={translate("repetitions")}
                      dataKey="reps"
                      stroke="#f00"
                      dot={{ fill: "#f00" }}
                      activeDot={{ stroke: "#f0f0f000" }}
                      isAnimationActive={false}
                    />
                    <Legend />
                  </LineChart>
                </ResponsiveContainer>
              </div>
              <br />
              <hr />
              <table>
                <colgroup>
                  <col style={{ width: "350px" }} />
                  <col style={{ width: "150px" }} />
                </colgroup>

                <thead>
                  <tr>
                    <th>{translate("workout")}</th>
                    <th>{translate("series")}</th>
                  </tr>
                </thead>
                <tbody>
                  {exercise.workouts.map((workout) =>
                    exercise.series[workout].map((serie, idx) => (
                      <tr key={`${workout}-${idx}`}>
                        {idx === 0 && (
                          <td
                            style={{
                              borderBottom:
                                idx === 0 ? "1px solid #e4e4e430" : "",
                            }}
                            rowSpan={exercise.series[workout].length}
                          >
                            {workout.split("\n")[0]}
                            <br />
                            {workout.split("\n")[1]}
                          </td>
                        )}

                        <td
                          style={{
                            borderBottom:
                              idx === exercise.series[workout].length - 1
                                ? "1px solid #e4e4e430"
                                : "",
                          }}
                        >
                          {serie.reps}x{serie.weight} Kg
                        </td>
                      </tr>
                    )),
                  )}
                </tbody>
              </table>
            </>
          )}
        </Modal.Body>
      </Modal>
    </div>
  );
}
