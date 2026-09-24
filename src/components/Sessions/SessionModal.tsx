import { LoadingContext } from "@/context/LoadingContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionSetsUpdate } from "@/utils/backend/models";
import { useContext, useState } from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  IconButton,
  TextareaAutosize,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { UnitUtils } from "@/utils/UnitUtils";
import { SessionFrontDetails } from "@/utils/SessionUtils";
import { SessionMap } from "./helpers/SessionMap";
import { SessionDetailsTable } from "./helpers/SessionDetailsTable";
import { SessionHeartRateChart } from "./helpers/SessionHeartRateChart";
import { SessionSetsTable } from "./helpers/SessionSetsTable";
import "@/styles/Sessions/SessionModal.css";

type Props = {
  session: SessionFrontDetails;
  onClose: () => void;
  onUpdate: () => void;
};

export function SessionModal({ session, onClose, onUpdate }: Props) {
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);
  const [originalSession] = useState(session);
  const [localSession, setLocalSession] = useState({ ...session });
  const [changed, setChanged] = useState(false);
  const [sets, setSets] = useState(session.sets.slice());
  const [notes, setNotes] = useState(session.notes);

  const hasChanges = (sets: typeof localSession.sets) =>
    sets.some(
      (serie, idx) =>
        serie.reps != originalSession.sets[idx].reps ||
        serie.weight != originalSession.sets[idx].weight,
    );

  const updateSerie = (
    exercise: string,
    idx: number,
    field: "reps" | "weight",
    value: number,
  ) => {
    setLocalSession((prev) => {
      const serieIdx = prev.grouped_series[exercise][idx].idx;

      const groupedForEx = prev.grouped_series[exercise].slice();
      groupedForEx[idx] = { ...groupedForEx[idx], [field]: value };

      const sets = prev.sets.slice();
      sets[serieIdx] = { ...sets[serieIdx], [field]: value };
      setSets(sets);

      setChanged(hasChanges(sets) || notes != originalSession.notes);

      return {
        ...prev,
        sets,
        grouped_series: { ...prev.grouped_series, [exercise]: groupedForEx },
      };
    });
  };

  const updateNote = (notes: string) => {
    setLocalSession((prev) => {
      setNotes(notes);
      setChanged(hasChanges(sets) || notes != originalSession.notes);

      return {
        ...prev,
        notes,
      };
    });
  };

  const saveChanges = () => {
    startLoading();
    const update: SessionSetsUpdate = {
      timestamp: localSession.timestamp,
      sets: [],
      notes: notes.length > 0 ? notes : null,
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
            <SessionMap
              timestamp={localSession.timestamp}
              validPoints={localSession.valid_points}
              gpsSegments={localSession.gps_segments}
              startPoint={localSession.start_point}
              finishPoint={localSession.finish_point}
              laps={localSession.laps}
            />
            <hr />
          </>
        )}
        <SessionDetailsTable session={localSession} />
        {localSession.hrBreathData.length > 0 && (
          <>
            <hr />
            <SessionHeartRateChart
              hrBreathData={localSession.hrBreathData}
              hrRanges={localSession.hrRanges}
              zonesTimes={localSession.zones_times}
              totalElapsedTime={localSession.total_elapsed_time}
            />
            <br />
          </>
        )}
        {localSession.sets && Object.keys(localSession.sets).length > 0 && (
          <SessionSetsTable
            exercises={localSession.exercises}
            groupedSeries={localSession.grouped_series}
            onUpdateSerie={updateSerie}
          />
        )}
        <div>
          <TextareaAutosize
            minRows={4}
            placeholder={translate("session_notes")}
            value={notes}
            style={{
              width: "100%",
              backgroundColor: "#202020",
              color: "white",
              resize: "none",
            }}
            onChange={(e) => {
              updateNote(e.target.value);
            }}
          />
        </div>
        <div className="session-sets-actions">
          <Button
            id="import-button"
            variant="contained"
            disabled={!changed}
            className="full-width-button"
            onClick={saveChanges}
          >
            {translate("update_session")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
