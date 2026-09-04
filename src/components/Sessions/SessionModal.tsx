import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { TimeUtils } from "@/utils/TimeUtils";
import { SessionSeriesUpdate } from "@/utils/backend/models";
import L from "leaflet";
import { useContext, useState } from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  FormControl,
  FormControlLabel,
  IconButton,
  Radio,
  RadioGroup,
  TextField,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
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
import { SessionFrontDetails } from "@/utils/SessionUtils";

type Props = {
  session: SessionFrontDetails;
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
const urls = [
  "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
  "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}",
];

export function SessionModal({ session, onClose, onUpdate }: Props) {
  const { startLoading, finishLoading, translate, settings } =
    useContext(AppContext);
  const [originalSession] = useState(session);
  const [localSession, setLocalSession] = useState({ ...session });
  const [changed, setChanged] = useState(false);
  const [url, setUrl] = useState(0);

  const handleMapTypeChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setUrl(event.target.value === "street" ? 0 : 1);
  };

  const updateSerieReps = (exercise: string, idx: number, newVal: string) => {
    let reps = parseInt(newVal);
    if (isNaN(reps)) {
      reps = 0;
    }
    const newObj = structuredClone(localSession);
    const serieIdx = newObj.grouped_series[exercise][idx].idx;
    newObj.grouped_series[exercise][idx].reps = reps;
    newObj.series[serieIdx].reps = reps;
    setLocalSession(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(session));
  };

  const updateSerieWeight = (exercise: string, idx: number, newVal: string) => {
    let weight = parseFloat(newVal);
    if (isNaN(weight)) {
      weight = 0;
    }
    const newObj = structuredClone(localSession);
    const serieIdx = newObj.grouped_series[exercise][idx].idx;
    newObj.grouped_series[exercise][idx].weight = weight;
    newObj.series[serieIdx].weight = weight;
    setLocalSession(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(session));
  };

  const saveChanges = () => {
    startLoading();
    const update: SessionSeriesUpdate = {
      timestamp: localSession.timestamp,
      series: [],
    };
    localSession.series.forEach((serie, serIdx) => {
      if (
        originalSession.series[serIdx].reps != serie.reps ||
        originalSession.series[serIdx].weight != serie.weight
      ) {
        update.series.push({
          ...serie,
          weight: UnitUtils.toKg(serie.weight, settings.weight_unit),
        });
      }
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
    <Dialog open={true} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>
        {localSession.sport}
        <>
          {localSession.name.length > 0 && (
            <span>
              : <span style={{ marginLeft: "10px" }}>{localSession.name}</span>
            </span>
          )}
        </>
        <IconButton
          onClick={onClose}
          sx={{ position: "absolute", right: 8, top: 8 }}
        >
          <CloseIcon />
        </IconButton>
      </DialogTitle>

      <DialogContent dividers>
        {localSession.valid_points.length > 0 && (
          <>
            <MapContainer
              bounds={localSession.valid_points}
              boundsOptions={{ padding: [20, 20] }}
              attributionControl={false}
              style={{ height: "200px", width: "100%" }}
            >
              <TileLayer url={urls[url]} attribution={""} />

              {localSession.gps_segments.map((val, idx) => (
                <Polyline
                  key={"segment-" + idx}
                  positions={val.coordinates}
                  color={val.color}
                  weight={4}
                />
              ))}

              <Marker
                position={localSession.start_point}
                icon={startIcon}
              ></Marker>

              <Marker
                position={localSession.finish_point}
                icon={endIcon}
              ></Marker>
            </MapContainer>
            <FormControl
              style={{ width: "100%", display: "flex", alignItems: "center" }}
            >
              <RadioGroup
                row
                name="row-radio-buttons-group"
                value={url === 0 ? "street" : "satellite"}
                onChange={handleMapTypeChange}
              >
                <FormControlLabel
                  value="street"
                  control={<Radio />}
                  label={translate("street_map")}
                />
                <FormControlLabel
                  value="satellite"
                  control={<Radio />}
                  label={translate("satellite_map")}
                />
              </RadioGroup>
            </FormControl>
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
                {localSession.total_calories - localSession.metabolic_calories}{" "}
                Kcal
              </td>
            </tr>
            <tr>
              <td>{translate("workout_load")}:</td>
              <td>{localSession.training_load}</td>
            </tr>
            {localSession.distance > 0 && (
              <>
                <tr>
                  <td>{translate("distance")}:</td>
                  <td>
                    {UnitUtils.fromKm(
                      localSession.distance,
                      settings.distance_unit,
                    ).toFixed(2)}{" "}
                    {UnitUtils.getUnit(settings.distance_unit)}
                  </td>
                </tr>
                <tr>
                  <td>{translate("speed")}:</td>
                  <td>
                    {UnitUtils.fromKm(
                      localSession.speed,
                      settings.distance_unit,
                    ).toFixed(2)}{" "}
                    {UnitUtils.getUnit(settings.distance_unit)}/h
                  </td>
                </tr>
                <tr>
                  <td>{translate("pace")}:</td>
                  <td>
                    {UnitUtils.fromKm(
                      localSession.pace,
                      settings.distance_unit,
                    ).toFixed(2)}{" "}
                    min/{UnitUtils.getUnit(settings.distance_unit)}
                  </td>
                </tr>
              </>
            )}
            {localSession.volume > 0 && (
              <tr>
                <td>{translate("volume")}:</td>
                <td>
                  {UnitUtils.fromKg(
                    localSession.volume,
                    settings.weight_unit,
                  ).toFixed(1)}{" "}
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
        {localSession.hrBreathData.length > 0 && (
          <>
            <hr />
            <div style={{ width: "100%", height: 200 }}>
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart
                  data={localSession.hrBreathData}
                  margin={{ top: 5, right: 5, left: 5, bottom: 5 }}
                >
                  <defs>
                    <linearGradient id="hrColor" x1="0" y1="0" x2="1" y2="0">
                      {localSession.hrBreathData.flatMap((point, i) => {
                        const start =
                          (i / localSession.hrBreathData.length) * 100;
                        const end =
                          ((i + 1) / localSession.hrBreathData.length) * 100;
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
                      <div style={{ textAlign: "center", fontWeight: "bold" }}>
                        {translate("heart_rate")}
                      </div>
                    )}
                  />
                  <CartesianGrid stroke="#80808000" strokeDasharray="5 5" />
                  <XAxis dataKey="idx" tick={false} />
                  <YAxis
                    width="auto"
                    domain={[
                      (4 * localSession.hrRanges[0]) / 5,
                      localSession.hrRanges[2],
                    ]}
                    ticks={localSession.hrRanges}
                  />
                  {localSession.hrRanges.map((val, idx) => {
                    return (
                      <ReferenceLine
                        key={"range-" + idx}
                        y={val}
                        stroke="white"
                        strokeDasharray="3 3"
                      />
                    );
                  })}
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
                        {TimeUtils.formatDuration(localSession.zones_times[0])}
                      </td>
                    </tr>
                    <tr>
                      <td>
                        {translate("hr_zone_2")}:{" "}
                        {TimeUtils.formatDuration(localSession.zones_times[1])}
                      </td>
                    </tr>
                    <tr>
                      <td>
                        {translate("hr_zone_3")}:{" "}
                        {TimeUtils.formatDuration(localSession.zones_times[2])}
                      </td>
                    </tr>
                  </table>
                </td>
                <td>
                  <table>
                    <tr>
                      <td>
                        {translate("hr_zone_4")}:{" "}
                        {TimeUtils.formatDuration(localSession.zones_times[3])}
                      </td>
                    </tr>
                    <tr>
                      <td>
                        {translate("hr_zone_5")}:{" "}
                        {TimeUtils.formatDuration(localSession.zones_times[4])}
                      </td>
                    </tr>
                  </table>
                </td>
              </tr>
            </table>
            <br />
          </>
        )}
        {localSession.series && Object.keys(localSession.series).length > 0 && (
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
                  localSession.grouped_series[exercise].map((serie, idx) => (
                    <tr key={`${exercise}-${idx}`}>
                      {idx === 0 && (
                        <td
                          style={{
                            borderBottom:
                              idx === 0 ? "1px solid #e4e4e430" : "",
                          }}
                          rowSpan={localSession.grouped_series[exercise].length}
                        >
                          {exercise}
                        </td>
                      )}

                      <td
                        style={{
                          borderBottom:
                            idx ===
                            localSession.grouped_series[exercise].length - 1
                              ? "1px solid #e4e4e430"
                              : "",
                          paddingBottom:
                            idx ===
                            localSession.grouped_series[exercise].length - 1
                              ? "5px"
                              : "",
                          paddingTop: idx === 0 ? "5px" : "",
                        }}
                      >
                        <TextField
                          variant="standard"
                          type="number"
                          value={serie.reps}
                          slotProps={{
                            htmlInput: {
                              className: "no-spinner",
                              min: 0,
                              style: { width: "2em", textAlign: "center" },
                            },
                          }}
                          onChange={(e) => {
                            updateSerieReps(exercise, idx, e.target.value);
                          }}
                        />{" "}
                        x{" "}
                        <TextField
                          variant="standard"
                          type="number"
                          value={serie.weight?.toString()}
                          slotProps={{
                            htmlInput: {
                              className: "no-spinner",
                              min: 0,
                              style: { width: "3.5em", textAlign: "center" },
                            },
                          }}
                          onChange={(e) => {
                            updateSerieWeight(exercise, idx, e.target.value);
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
                variant="contained"
                disabled={!changed}
                style={{ width: "100%" }}
                onClick={saveChanges}
              >
                {translate("update_sets")}
              </Button>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
