import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { TimeUtils } from "@/utils/TimeUtils";
import { SessionDetails, SessionSeriesUpdate } from "@/utils/backend/models";
import L from "leaflet";
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
import { UnitUtils } from "@/utils/UnitUtils";

type Props = {
  session: SessionDetails;
  onClose: () => void;
  onUpdate: () => void;
};

const makeMarkerIcon = (color: string) =>
  L.divIcon({
    className: "",
    html: `<svg width="25" height="41" viewBox="0 0 25 41" xmlns="http://www.w3.org/2000/svg">
      <path d="M12.5 0C5.6 0 0 5.6 0 12.5c0 9.4 12.5 28.5 12.5 28.5S25 21.9 25 12.5C25 5.6 19.4 0 12.5 0z" fill="${color}" stroke="white" stroke-width="1.5"/>
      <circle cx="12.5" cy="12.5" r="4.5" fill="white"/>
    </svg>`,
    iconSize: [25, 41],
    iconAnchor: [12, 41],
    popupAnchor: [1, -34],
  });

const startIcon = makeMarkerIcon("#2ecc71");
const endIcon = makeMarkerIcon("#e74c3c");

export function SessionModal({ session, onClose, onUpdate }: Props) {
  const { startLoading, finishLoading, translate, settings } =
    useContext(AppContext);
  const [originalSession] = useState(session);
  const [localSession, setLocalSession] = useState({ ...session });
  const [changed, setChanged] = useState(false);
  const [minHr, setMinHr] = useState(0);
  const [maxHr, setMaxHr] = useState(0);
  const [avgHr, setAvgHr] = useState(0);
  const [distance, setDistance] = useState(0);
  const [volume, setVolume] = useState(0);
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

    if (Object.entries(session.series).length > 0) {
      const localLocalSession = { ...session };
      Object.keys(localLocalSession.series).forEach((key) => {
        localLocalSession.series[key].forEach((_, idx) => {
          const copy = { ...localLocalSession.series[key][idx] };
          copy.weight = Number(
            UnitUtils.fromKg(copy.weight, settings.weight_unit).toFixed(1),
          );
          localLocalSession.series[key][idx] = copy;
        });
      });
      setLocalSession(localLocalSession);

      let vol = 0;
      Object.entries(localLocalSession.series).map(([, series]) => {
        series.forEach((serie) => {
          vol += serie.reps * serie.weight!;
        });
      });
      setVolume(vol);
    }

    const maxHr = Math.max(189, Math.max(...session.heart_rates));
    setMaxHr(Math.max(...session.heart_rates));
    const avgHr = Math.round(
      session.heart_rates.reduce((acc, valor) => acc + valor, 0) /
        session.heart_rates.length,
    );
    setAvgHr(avgHr);
    setMinHr(Math.min(...session.heart_rates));

    if (session.heart_rates.length > 0) {
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
          avg: avgHr,
          color,
        });
      });
    }
    setHrData(hrData);

    if (session.gps_coordinates.length > 0) {
      function toRadians(grados: number): number {
        return (grados * Math.PI) / 180;
      }

      let dist = 0.0;
      for (let i = 0; i < session.gps_coordinates.length - 1; i++) {
        const p1 = session.gps_coordinates[i];
        const p2 = session.gps_coordinates[i + 1];

        const lat1 = toRadians(p1[0]);
        const lat2 = toRadians(p2[0]);
        const dLat = toRadians(p2[0] - p1[0]);
        const dLon = toRadians(p2[1] - p1[1]);

        const a =
          Math.sin(dLat / 2) ** 2 +
          Math.cos(lat1) * Math.cos(lat2) * Math.sin(dLon / 2) ** 2;
        const c = 2 * Math.asin(Math.sqrt(a));

        dist += 6371 * c;
      }
      setDistance(dist);
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
          update.series.push({
            ...serie,
            weight: UnitUtils.toKg(serie.weight, settings.weight_unit),
          });
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
                boundsOptions={{ padding: [20, 20] }}
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

                <Marker
                  position={localSession.gps_coordinates[0]}
                  icon={startIcon}
                ></Marker>

                <Marker
                  position={
                    localSession.gps_coordinates[
                      localSession.gps_coordinates.length - 1
                    ]
                  }
                  icon={endIcon}
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
                <td>{TimeUtils.formatTimeDate(localSession.timestamp)}</td>
              </tr>
              <tr>
                <td>{translate("total_time")}:</td>
                <td>
                  {TimeUtils.formatDuration(localSession.total_elapsed_time)}
                </td>
              </tr>
              {localSession.active_time > 0 && (
                <tr>
                  <td>{translate("active_time")}:</td>
                  <td>{TimeUtils.formatDuration(localSession.active_time)}</td>
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
              {distance > 0 && (
                <>
                  <tr>
                    <td>{translate("distance")}:</td>
                    <td>
                      {UnitUtils.fromKm(
                        distance,
                        settings.distance_unit,
                      ).toFixed(2)}{" "}
                      {UnitUtils.getUnit(settings.distance_unit)}
                    </td>
                  </tr>
                  <tr>
                    <td>{translate("speed")}:</td>
                    <td>
                      {UnitUtils.fromKm(
                        distance / (localSession.total_elapsed_time / 3600),
                        settings.distance_unit,
                      ).toFixed(2)}{" "}
                      {UnitUtils.getUnit(settings.distance_unit)}/h
                    </td>
                  </tr>
                  <tr>
                    <td>{translate("pace")}:</td>
                    <td>
                      {UnitUtils.fromKm(
                        localSession.total_elapsed_time / 60 / distance,
                        settings.distance_unit,
                      ).toFixed(2)}{" "}
                      min/{UnitUtils.getUnit(settings.distance_unit)}
                    </td>
                  </tr>
                </>
              )}
              {volume > 0 && (
                <tr>
                  <td>{translate("volume")}:</td>
                  <td>
                    {UnitUtils.fromKg(volume, settings.weight_unit).toFixed(1)}{" "}
                    {UnitUtils.getUnit(settings.weight_unit)}
                  </td>
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
                      domain={[(4 * minHr) / 5, maxHr]}
                      ticks={[minHr, avgHr, maxHr]}
                    />
                    <ReferenceLine
                      y={avgHr}
                      stroke="white"
                      strokeDasharray="3 3"
                    />
                    <ReferenceLine
                      y={minHr}
                      stroke="white"
                      strokeDasharray="3 3"
                    />
                    <ReferenceLine
                      y={maxHr}
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
                          {TimeUtils.formatDuration(
                            localSession.zones_times[0],
                          )}
                        </td>
                      </tr>
                      <tr>
                        <td>
                          {translate("hr_zone_2")}:{" "}
                          {TimeUtils.formatDuration(
                            localSession.zones_times[1],
                          )}
                        </td>
                      </tr>
                      <tr>
                        <td>
                          {translate("hr_zone_3")}:{" "}
                          {TimeUtils.formatDuration(
                            localSession.zones_times[2],
                          )}
                        </td>
                      </tr>
                    </table>
                  </td>
                  <td>
                    <table>
                      <tr>
                        <td>
                          {translate("hr_zone_4")}:{" "}
                          {TimeUtils.formatDuration(
                            localSession.zones_times[3],
                          )}
                        </td>
                      </tr>
                      <tr>
                        <td>
                          {translate("hr_zone_5")}:{" "}
                          {TimeUtils.formatDuration(
                            localSession.zones_times[4],
                          )}
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
                              style={{ width: "3.5em", textAlign: "center" }}
                              onChange={(e) => {
                                updateSerieWeight(
                                  exercise,
                                  idx,
                                  e.target.value,
                                );
                              }}
                            />
                            {" " + UnitUtils.getUnit(settings.weight_unit)}
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
