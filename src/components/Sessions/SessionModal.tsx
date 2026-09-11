import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { TimeUtils } from "@/utils/TimeUtils";
import { SessionSetsUpdate } from "@/utils/backend/models";
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
import { SessionFrontDetails, SessionUtils } from "@/utils/SessionUtils";
import EmojiEventsIcon from "@mui/icons-material/EmojiEvents";
import "@/styles/Sessions/SessionModal.css";

type Props = {
  session: SessionFrontDetails;
  onClose: () => void;
  onUpdate: () => void;
};

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
  const [url, setUrl] = useState(1);

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
    newObj.sets[serieIdx].reps = reps;
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
    newObj.sets[serieIdx].weight = weight;
    setLocalSession(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(session));
  };

  const saveChanges = () => {
    startLoading();
    const update: SessionSetsUpdate = {
      timestamp: localSession.timestamp,
      sets: [],
    };
    localSession.sets.forEach((serie, serIdx) => {
      if (
        originalSession.sets[serIdx].reps != serie.reps ||
        originalSession.sets[serIdx].weight != serie.weight
      ) {
        update.sets.push({
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
        {translate("sport_" + localSession.sport) +
          " - " +
          translate(
            "sport_" + localSession.sport + "_" + localSession.sub_sport,
          )}
        {localSession.name.length > 0 && (
          <span>
            : <span className="session-modal-name">{localSession.name}</span>
          </span>
        )}
        <IconButton onClick={onClose} className="modal-close-button">
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
              className="session-map"
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
                icon={SessionUtils.START_ICON}
              ></Marker>

              <Marker
                position={localSession.finish_point}
                icon={SessionUtils.END_ICON}
              ></Marker>

              {localSession.laps
                .filter((lap) => lap.start_latitude && lap.start_longitude)
                .map((lap) =>
                  SessionUtils.makeLapMarker(lap.idx + 1, [
                    lap.start_latitude!,
                    lap.start_longitude!,
                  ]),
                )}
            </MapContainer>
            <FormControl className="session-map-type-control">
              <RadioGroup
                row
                name="row-radio-buttons-group"
                value={url === 0 ? "street" : "satellite"}
                onChange={handleMapTypeChange}
              >
                <FormControlLabel
                  value="satellite"
                  control={<Radio />}
                  label={translate("satellite_map")}
                />
                <FormControlLabel
                  value="street"
                  control={<Radio />}
                  label={translate("street_map")}
                />
              </RadioGroup>
            </FormControl>
            <hr />
          </>
        )}
        <table id="session-details-table">
          <colgroup>
            <col className="col-200" />
            <col className="col-250" />
            <col />
          </colgroup>
          <tbody>
            <tr>
              <td>{translate("date")}:</td>
              <td>{TimeUtils.formatTimeDate(localSession.timestamp)}</td>
            </tr>
            <tr>
              <td>{translate("time")}:</td>
              <td>
                {(localSession.active_time > 0
                  ? TimeUtils.formatDuration(localSession.active_time) + " / "
                  : "") +
                  TimeUtils.formatDuration(localSession.total_elapsed_time)}
              </td>
            </tr>
            <tr>
              <td>{translate("calories")}:</td>
              <td>
                {localSession.total_calories -
                  localSession.metabolic_calories +
                  " / " +
                  localSession.total_calories}{" "}
                Kcal
              </td>
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
                      localSession.distance /
                        (localSession.total_elapsed_time / 3600),
                      settings.distance_unit,
                    ).toFixed(2)}{" "}
                    {UnitUtils.getUnit(settings.distance_unit)}/h (
                    {TimeUtils.formatDuration(
                      UnitUtils.toKm(
                        localSession.total_elapsed_time / localSession.distance,
                        settings.distance_unit,
                      ),
                    )}{" "}
                    min/{UnitUtils.getUnit(settings.distance_unit)}){" "}
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
            <tr>
              <td>{translate("workout_load")}:</td>
              <td>{localSession.training_load}</td>
            </tr>
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
            <div className="session-hr-chart">
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
                      <div className="text-center">
                        <div className="session-hr-legend-heading">
                          {translate("heart_rate")}
                        </div>
                        <div className="session-hr-legend-zones">
                          {localSession.zones_times[1] > 0 && (
                            <span className="session-hr-zone session-hr-zone-1">
                              {TimeUtils.formatDuration(
                                localSession.zones_times[0],
                              ) +
                                " (" +
                                Math.round(
                                  100 *
                                    (localSession.zones_times[0] /
                                      localSession.total_elapsed_time),
                                ) +
                                "%)"}
                            </span>
                          )}
                          {localSession.zones_times[1] > 0 && (
                            <span className="session-hr-zone session-hr-zone-2">
                              {TimeUtils.formatDuration(
                                localSession.zones_times[1],
                              ) +
                                " (" +
                                Math.round(
                                  100 *
                                    (localSession.zones_times[1] /
                                      localSession.total_elapsed_time),
                                ) +
                                "%)"}
                            </span>
                          )}
                          {localSession.zones_times[2] > 0 && (
                            <span className="session-hr-zone session-hr-zone-3">
                              {TimeUtils.formatDuration(
                                localSession.zones_times[2],
                              ) +
                                " (" +
                                Math.round(
                                  100 *
                                    (localSession.zones_times[2] /
                                      localSession.total_elapsed_time),
                                ) +
                                "%)"}
                            </span>
                          )}
                          {localSession.zones_times[3] > 0 && (
                            <span className="session-hr-zone session-hr-zone-4">
                              {TimeUtils.formatDuration(
                                localSession.zones_times[3],
                              ) +
                                " (" +
                                Math.round(
                                  100 *
                                    (localSession.zones_times[3] /
                                      localSession.total_elapsed_time),
                                ) +
                                "%)"}
                            </span>
                          )}
                          {localSession.zones_times[4] > 0 && (
                            <span className="session-hr-zone session-hr-zone-5">
                              {TimeUtils.formatDuration(
                                localSession.zones_times[4],
                              ) +
                                " (" +
                                Math.round(
                                  100 *
                                    (localSession.zones_times[4] /
                                      localSession.total_elapsed_time),
                                ) +
                                "%)"}
                            </span>
                          )}
                        </div>
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
            <br />
          </>
        )}
        {localSession.sets && Object.keys(localSession.sets).length > 0 && (
          <div className="session-sets-container">
            <table>
              <colgroup>
                <col className="col-350" />
                <col className="col-200" />
              </colgroup>

              <thead>
                <tr className="divider-bottom">
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
                          className="divider-bottom"
                          rowSpan={localSession.grouped_series[exercise].length}
                        >
                          {translate(
                            "exercise_" + serie.ex_cat + "_" + serie.ex_id,
                          )}
                        </td>
                      )}

                      <td
                        className={[
                          idx ===
                          localSession.grouped_series[exercise].length - 1
                            ? "divider-bottom group-cell-last"
                            : undefined,
                          idx === 0 ? "group-cell-first" : undefined,
                        ]
                          .filter(Boolean)
                          .join(" ")}
                      >
                        <TextField
                          variant="standard"
                          type="number"
                          value={serie.reps}
                          slotProps={{
                            htmlInput: {
                              className: "no-spinner session-sets-reps-input",
                              min: 0,
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
                              className: "no-spinner session-sets-weight-input",
                              min: 0,
                            },
                          }}
                          onChange={(e) => {
                            updateSerieWeight(exercise, idx, e.target.value);
                          }}
                        />
                        {" " + UnitUtils.getUnit(settings.weight_unit) + " "}
                        {serie.pr && (
                          <EmojiEventsIcon className="trophy-icon" />
                        )}
                      </td>
                    </tr>
                  )),
                )}
              </tbody>
            </table>
            <div className="session-sets-actions">
              <Button
                id="import-button"
                variant="contained"
                disabled={!changed}
                className="full-width-button"
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
