import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionDetails, SessionSeriesUpdate } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { Button, Modal } from "react-bootstrap";
import { MapContainer, Marker, Polyline, TileLayer } from "react-leaflet";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ReferenceLine,
  ResponsiveContainer,
  XAxis,
  YAxis,
  Legend,
} from "recharts";

type Props = {
  session: SessionDetails;
  onClose: () => void;
  onUpdate: () => void;
};

export function SessionModal({ session, onClose, onUpdate }: Props) {
  const { startLoading, finishLoading, translate } = useContext(AppContext);
  const [originalSession] = useState(session);
  const [localSession, setLocalSession] = useState({ ...session });
  const [changed, setChanged] = useState(false);
  const [minHr, setMinHr] = useState(0);
  const [hrData, setHrData] = useState<
    {
      idx: number;
      hr: number;
      avg: number;
      color: string;
    }[]
  >([]);

  useEffect(() => {
    const hrData: {
      idx: number;
      hr: number;
      avg: number;
      color: string;
    }[] = [];

    const maxHr = Math.max(189, session.max_heart_rate);

    if (session.heart_rates && session.heart_rates.length > 0) {
      session.heart_rates.forEach((hr, idx) => {
        let color = "red";

        const rateVal = (hr * 1.0) / (maxHr * 1.0);
        if (rateVal <= 0.6) {
          color = "gray";
        } else if (rateVal <= 0.7) {
          color = "turquoise";
        } else if (rateVal <= 0.8) {
          color = "green";
        } else if (rateVal <= 0.9) {
          color = "orange";
        }
        hrData.push({
          idx: idx * 2,
          hr,
          avg: session.avg_heart_rate,
          color,
        });
      });
    }

    setHrData(hrData);
    if (session.heart_rates && session.heart_rates.length > 0) {
      setMinHr(Math.min(...session.heart_rates));
    }
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
    Object.entries(localSession.series).forEach(([ex, series]) => {
      series.forEach((serie, serIdx) => {
        if (
          originalSession.series[ex][serIdx].reps != serie.reps ||
          originalSession.series[ex][serIdx].weight != serie.weight
        ) {
          update.series.push(serie);
        }
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
            {localSession.sport}
            <>
              {localSession.name.length > 0 && (
                <span>
                  :{" "}
                  <span style={{ marginLeft: "10px" }}>
                    {localSession.name}
                  </span>
                </span>
              )}
            </>
          </Modal.Title>
        </Modal.Header>

        <Modal.Body>
          {localSession.gps_coordinates.length > 0 && (
            <>
              <MapContainer
                bounds={localSession.gps_coordinates}
                boundsOptions={{ padding: [25, 25] }}
                attributionControl={false}
                style={{ height: "200px", width: "100%" }}
              >
                <TileLayer
                  url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                  attribution={""}
                />

                <Polyline
                  positions={localSession.gps_coordinates}
                  color="blue"
                  weight={4}
                />

                <Marker position={localSession.gps_coordinates[0]}></Marker>

                <Marker
                  position={
                    localSession.gps_coordinates[
                      localSession.gps_coordinates.length - 1
                    ]
                  }
                ></Marker>
              </MapContainer>
              <hr />
            </>
          )}
          <table id="session-details-table">
            <colgroup>
              <col style={{ width: "200px" }} />
              <col style={{ width: "150px" }} />
              <col />
            </colgroup>
            <tbody>
              <tr>
                <td>{translate("date")}:</td>
                <td>{localSession.date}</td>
              </tr>
              <tr>
                <td>{translate("total_time")}:</td>
                <td>{localSession.total_elapsed_time}</td>
              </tr>
              {localSession.active_time && (
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
                <td>{translate("workout_load")}:</td>
                <td>{localSession.training_load}</td>
              </tr>
              {getVolume() > 0 && (
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
          {hrData.length > 0 && (
            <>
              <hr />
              <div style={{ width: "100%", height: 200 }}>
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart
                    data={hrData}
                    margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
                  >
                    <defs>
                      <linearGradient id="hrColor" x1="0" y1="0" x2="1" y2="0">
                        {hrData.flatMap((point, i) => {
                          const start = (i / hrData.length) * 100;
                          const end = ((i + 1) / hrData.length) * 100;
                          return [
                            <stop
                              key={`${i}-start`}
                              offset={`${start}%`}
                              stopColor={point.color}
                            />,
                            <stop
                              key={`${i}-end`}
                              offset={`${end}%`}
                              stopColor={point.color}
                            />,
                          ];
                        })}
                      </linearGradient>
                    </defs>
                    <Legend
                      position={"top"}
                      content={() => (
                        <div
                          style={{ textAlign: "center", fontWeight: "bold" }}
                        >
                          {translate("heart_rate")}
                        </div>
                      )}
                    />
                    <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
                    <XAxis dataKey="idx" tick={false} />
                    <YAxis
                      width="auto"
                      domain={[(4 * minHr) / 5, session.max_heart_rate]}
                      ticks={[
                        minHr,
                        session.avg_heart_rate,
                        session.max_heart_rate,
                      ]}
                    />
                    <ReferenceLine
                      y={session.avg_heart_rate}
                      stroke="white"
                      strokeDasharray="3 3"
                    />
                    <ReferenceLine
                      y={minHr}
                      stroke="white"
                      strokeDasharray="3 3"
                    />
                    <ReferenceLine
                      y={session.max_heart_rate}
                      stroke="white"
                      strokeDasharray="3 3"
                    />
                    <Area
                      dataKey="hr"
                      type="monotone"
                      isAnimationActive={false}
                      stroke="url(#hrColor)"
                      fill="url(#hrColor)"
                      fillOpacity={1}
                      activeDot={false}
                    />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
              <table style={{ position: "relative", top: "-20px" }}>
                <colgroup>
                  <col style={{ width: "250px" }} />
                  <col style={{ width: "250px" }} />
                </colgroup>
                <tr>
                  <td>
                    <table>
                      <tr>
                        <td>
                          {translate("hr_zone_1")}:{" "}
                          {localSession.zones_times[0]}
                        </td>
                      </tr>
                      <tr>
                        <td>
                          {translate("hr_zone_2")}:{" "}
                          {localSession.zones_times[1]}
                        </td>
                      </tr>
                      <tr>
                        <td>
                          {translate("hr_zone_3")}:{" "}
                          {localSession.zones_times[2]}
                        </td>
                      </tr>
                    </table>
                  </td>
                  <td>
                    <table>
                      <tr>
                        <td>
                          {translate("hr_zone_4")}:{" "}
                          {localSession.zones_times[3]}
                        </td>
                      </tr>
                      <tr>
                        <td>
                          {translate("hr_zone_5")}:{" "}
                          {localSession.zones_times[4]}
                        </td>
                      </tr>
                    </table>
                  </td>
                </tr>
              </table>
              <br />
            </>
          )}
          {localSession.series &&
            Object.keys(localSession.series).length > 0 && (
              <div style={{ position: "relative", top: "-20px" }}>
                <table>
                  <colgroup>
                    <col style={{ width: "350px" }} />
                    <col style={{ width: "150px" }} />
                  </colgroup>

                  <thead>
                    <tr style={{ borderBottom: "1px solid #e4e4e430" }}>
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
              </div>
            )}
        </Modal.Body>
      </Modal>
    </div>
  );
}
