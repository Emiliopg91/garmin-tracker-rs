import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { LoadingContext } from "@/context/LoadingContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionLap } from "@/utils/backend/models";
import { SessionUtils } from "@/utils/SessionUtils";
import { useContext, useState } from "react";
import { FormControl, FormControlLabel, Link, Radio, RadioGroup } from "@mui/material";
import { MapContainer, Marker, Polyline, TileLayer } from "react-leaflet";

const urls = [
  "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
  "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}",
];

type Props = {
  timestamp: number;
  validPoints: [number, number][];
  gpsSegments: { coordinates: [[number, number], [number, number]]; color: string }[];
  startPoint: [number, number];
  finishPoint: [number, number];
  laps: SessionLap[];
};

export function SessionMap({
  timestamp,
  validPoints,
  gpsSegments,
  startPoint,
  finishPoint,
  laps,
}: Props) {
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate } = useContext(I18nSettingsContext);
  const [url, setUrl] = useState(1);

  const handleMapTypeChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setUrl(event.target.value === "street" ? 0 : 1);
  };

  const exportTrack = () => {
    startLoading();
    BackendClient.exportGpx(timestamp).finally(() => {
      finishLoading();
    });
  };

  return (
    <>
      <MapContainer
        bounds={validPoints}
        boundsOptions={{ padding: [20, 20] }}
        attributionControl={false}
        className="session-map"
      >
        <TileLayer url={urls[url]} attribution={""} />

        {gpsSegments.map((val, idx) => (
          <Polyline
            key={"segment-" + idx}
            positions={val.coordinates}
            color={val.color}
            weight={4}
          />
        ))}

        <Marker position={startPoint} icon={SessionUtils.START_ICON}></Marker>

        <Marker
          position={finishPoint}
          icon={SessionUtils.END_ICON}
        ></Marker>

        {laps
          .filter((lap) => lap.start_latitude && lap.start_longitude)
          .map((lap) =>
            SessionUtils.makeLapMarker(lap.idx + 1, [
              lap.start_latitude!,
              lap.start_longitude!,
            ]),
          )}
      </MapContainer>
      <div style={{ display: "flex" }}>
        <FormControl className="session-map-type-control" style={{ flex: 1 }}>
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
        <div style={{ display: "block", margin: "auto" }}>
          <Link href="#" onClick={exportTrack} style={{ flex: 1 }}>
            {translate("export_gpx")}
          </Link>
        </div>
      </div>
    </>
  );
}
