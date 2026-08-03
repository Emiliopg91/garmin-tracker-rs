import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionDetails, SessionSeriesUpdate } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { Button, Modal } from "react-bootstrap";
import {
  CartesianGrid,
  ComposedChart,
  Legend,
  Line,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";

type Props = {
  session: SessionDetails;
  onClose: () => void;
  onUpdate: () => void;
};

export function SessionModal({ session, onClose, onUpdate }: Props) {
  const { startLoading, finishLoading, translate } = useContext(AppContext);
  const [localSession, setLocalSession] = useState({ ...session });
  const [changed, setChanged] = useState(false);
  const [hrData, setHrData] = useState<
    { idx: number; hr: number; avg: number }[]
  >([]);
  const [minHr, setMinHr] = useState(0);

  useEffect(() => {
    const hrData: { idx: number; hr: number; avg: number }[] = [];
    let minHr = 300;

    if (session.heart_rates && session.heart_rates.length > 0) {
      session.heart_rates.forEach((hr, idx) => {
        minHr = Math.min(minHr, hr);
        hrData.push({ idx, hr, avg: session.avg_heart_rate });
      });
    }

    setHrData(hrData);
    setMinHr(minHr);

    console.log(minHr);
  }, []);

  const updateSerieReps = (exercise: string, idx: number, newVal: string) => {
    let reps = parseInt(newVal);
    if (isNaN(reps)) {
      reps = 0;
    }
    const newObj = {
      ...localSession,
      series: {
        ...localSession.series,
        [exercise]: localSession.series[exercise].map((serie, id) =>
          id === idx ? { ...serie, reps } : serie,
        ),
      },
    };
    setLocalSession(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(session));
  };

  const updateSerieWeight = (exercise: string, idx: number, newVal: string) => {
    let weight = parseFloat(newVal);
    if (isNaN(weight)) {
      weight = 0;
    }
    const newObj = {
      ...localSession,
      series: {
        ...localSession.series,
        [exercise]: localSession.series[exercise].map((serie, id) =>
          id === idx ? { ...serie, weight } : serie,
        ),
      },
    };
    setLocalSession(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(session));
  };

  const getVolume = () => {
    let volume = 0;
    Object.entries(localSession.series).map(([, series]) => {
      series.forEach((serie) => {
        volume += serie.reps * serie.weight!;
      });
    });

    return volume;
  };

  const saveChanges = () => {
    startLoading();
    const update: SessionSeriesUpdate = {
      timestamp: localSession.timestamp,
      series: [],
    };
    Object.entries(localSession.series).forEach(([, series]) => {
      series.forEach((serie) => {
        update.series.push(serie);
      });
    });
    BackendClient.saveSessionChanges(update)
      .then(() => {
        onUpdate();
        onClose();
      })
      .finally(() => {
        finishLoading();
      });
  };

  return (
    <div
      className="modal show"
      style={{ display: "block", position: "initial" }}
    >
      <Modal show={true} onHide={onClose} data-bs-theme="dark">
        <Modal.Header closeButton>
          <Modal.Title>
            {session.sub_sport == "strength_training"
              ? localSession.name
              : translate("other")}
            <small style={{ fontSize: "17px", marginLeft: "30px" }}>
              {localSession.date}
            </small>
          </Modal.Title>
        </Modal.Header>

        <Modal.Body>
          {hrData.length > 0 && (
            <div style={{ width: "100%", height: 200 }}>
              <ResponsiveContainer width="100%" height="100%">
                <ComposedChart
                  data={hrData}
                  margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
                >
                  <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
                  <XAxis
                    dataKey="idx"
                    type="number"
                    stroke="#fff"
                    tick={false}
                    height={0}
                  />
                  <YAxis
                    stroke="#fff"
                    width={0}
                    domain={[(3 * minHr) / 4, session.max_heart_rate]}
                    tick={false}
                  />{" "}
                  <Line
                    type="monotone"
                    name={translate("heart_rate")}
                    dataKey="hr"
                    stroke="red"
                    dot={false}
                    isAnimationActive={false}
                    activeDot={false}
                  />
                  <Line
                    type="monotone"
                    name={translate("avg_heart_rate")}
                    dataKey="avg"
                    stroke="#ffffff40"
                    dot={false}
                    legendType="none"
                    isAnimationActive={false}
                    activeDot={false}
                  />
                  <Legend />
                </ComposedChart>
              </ResponsiveContainer>
            </div>
          )}
          <table id="session-details-table">
            <colgroup>
              <col style={{ width: "250px" }} />
              <col style={{ width: "150px" }} />
              <col />
            </colgroup>
            <tbody>
              <tr>
                <td>{translate("total_time")}:</td>
                <td>{localSession.total_elapsed_time}</td>
              </tr>
              {session.sub_sport == "strength_training" && (
                <tr>
                  <td>{translate("active_time")}:</td>
                  <td>{localSession.active_time}</td>
                </tr>
              )}
              <tr>
                <td>{translate("total_calories")}:</td>
                <td>{localSession.total_calories} Kcal</td>
              </tr>
              <tr>
                <td>{translate("active_calories")}:</td>
                <td>
                  {localSession.total_calories -
                    localSession.metabolic_calories}{" "}
                  Kcal
                </td>
              </tr>
              <tr>
                <td>{translate("avg_heart_rate")}:</td>
                <td> {localSession.avg_heart_rate} BPM</td>
              </tr>
              <tr>
                <td>{translate("max_heart_rate")}:</td>
                <td>{localSession.max_heart_rate} BPM</td>
              </tr>
              <tr>
                <td>{translate("workout_load")}:</td>
                <td>{localSession.training_load}</td>
              </tr>
              {session.sub_sport == "strength_training" && (
                <tr>
                  <td>{translate("volume")}:</td>
                  <td>{getVolume()} Kg</td>
                </tr>
              )}
              {localSession.device && (
                <tr>
                  <td>{translate("imported_from")}:</td>
                  <td>{localSession.device}</td>
                </tr>
              )}
            </tbody>
          </table>
          {session.sub_sport == "strength_training" &&
            Object.keys(localSession.series).length > 0 && (
              <>
                <hr />
                <table>
                  <colgroup>
                    <col style={{ width: "350px" }} />
                    <col style={{ width: "150px" }} />
                  </colgroup>

                  <thead>
                    <tr>
                      <th>{translate("exercise")}:</th>
                      <th>{translate("series")}:</th>
                    </tr>
                  </thead>
                  <tbody>
                    {localSession.exercises.map((exercise) =>
                      localSession.series[exercise].map((serie, idx) => (
                        <tr key={`${exercise}-${idx}`}>
                          {idx === 0 && (
                            <td
                              style={{
                                borderBottom:
                                  idx === 0 ? "1px solid #e4e4e430" : "",
                              }}
                              rowSpan={localSession.series[exercise].length}
                            >
                              {exercise}
                            </td>
                          )}

                          <td
                            style={{
                              borderBottom:
                                idx === localSession.series[exercise].length - 1
                                  ? "1px solid #e4e4e430"
                                  : "",
                              paddingBottom:
                                idx === localSession.series[exercise].length - 1
                                  ? "5px"
                                  : "",
                              paddingTop: idx === 0 ? "5px" : "",
                            }}
                          >
                            <input
                              type="number"
                              value={serie.reps}
                              className="no-spinner"
                              min={0}
                              style={{ width: "2em", textAlign: "center" }}
                              onChange={(e) => {
                                updateSerieReps(exercise, idx, e.target.value);
                              }}
                            />{" "}
                            x{" "}
                            <input
                              type="number"
                              value={serie.weight?.toString()}
                              className="no-spinner"
                              min={0}
                              style={{ width: "3em", textAlign: "center" }}
                              onChange={(e) => {
                                updateSerieWeight(
                                  exercise,
                                  idx,
                                  e.target.value,
                                );
                              }}
                            />
                            Kg
                          </td>
                        </tr>
                      )),
                    )}
                  </tbody>
                </table>
                <div style={{ padding: "5px" }}>
                  <Button
                    id="import-button"
                    disabled={!changed}
                    style={{ width: "100%" }}
                    onClick={saveChanges}
                  >
                    {translate("update_sets")}
                  </Button>
                </div>
              </>
            )}
        </Modal.Body>
      </Modal>
    </div>
  );
}
