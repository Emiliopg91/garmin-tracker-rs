import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { useContext, useEffect, useState } from "react";
import { Dialog, DialogContent, DialogTitle, IconButton } from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { SessionFrontDetails } from "@/utils/SessionUtils";

type Props = {
  sessions: SessionFrontDetails[];
  onClose: () => void;
};

export function SessionsCompareModal({ sessions, onClose }: Props) {
  const {
    translate,
    fromKg,
    getWeightUnit,
    formatDuration,
    formatTimeDate,
    formatNumber,
  } = useContext(I18nSettingsContext);
  const [exercises, setExercises] = useState<string[]>([]);

  useEffect(() => {
    const exercises: string[] = [];
    let idx = 0;
    while (true) {
      let any = false;
      for (let i = 0; i < sessions.length; i++) {
        const session = sessions[i];
        if (session.exercises.length > idx) {
          const exercise = session.exercises[idx];
          if (!exercises.includes(exercise)) {
            exercises.push(exercise);
          }

          any = true;
        }
      }

      if (any) {
        idx++;
      } else {
        break;
      }
    }
    setExercises(exercises);
  }, []);

  return (
    <Dialog open={true} onClose={onClose}>
      <DialogTitle>
        {translate("sessions_comparaison") + ": " + sessions[0].name}
        <IconButton onClick={onClose} className="modal-close-button">
          <CloseIcon />
        </IconButton>
      </DialogTitle>

      <DialogContent dividers>
        <table id="sessions-compare-table">
          <colgroup>
            <col className="col-230" />
            <col className="col-180" />
            <col className="col-180" />
          </colgroup>

          <tbody>
            <tr>
              <td style={{ borderBottom: "1px solid white" }}></td>
              <td style={{ borderBottom: "1px solid white" }}>
                <b>{formatTimeDate(sessions[0].timestamp)}</b>
              </td>
              <td style={{ borderBottom: "1px solid white" }}>
                <b>{formatTimeDate(sessions[1].timestamp)}</b>
              </td>
            </tr>
            <tr>
              <td>{translate("time")}</td>
              <td>
                {formatDuration(sessions[0].active_time) +
                  " / " +
                  formatDuration(sessions[0].total_elapsed_time)}
              </td>
              <td>
                {formatDuration(sessions[1].active_time) +
                  " / " +
                  formatDuration(sessions[1].total_elapsed_time)}
              </td>
            </tr>
            <tr>
              <td>{translate("calories")}</td>
              <td>
                {sessions[0].total_calories -
                  sessions[0].metabolic_calories +
                  " / " +
                  sessions[0].total_calories}{" "}
                Kcal
              </td>
              <td>
                {sessions[1].total_calories -
                  sessions[1].metabolic_calories +
                  " / " +
                  sessions[1].total_calories}{" "}
                Kcal
              </td>
            </tr>
            <tr>
              <td>{translate("volume")}</td>
              <td>
                {formatNumber(fromKg(sessions[0].volume), 1)} {getWeightUnit()}
              </td>
              <td>
                {formatNumber(fromKg(sessions[1].volume), 1)} {getWeightUnit()}
              </td>
            </tr>
            <tr>
              <td>{translate("workout_load")}</td>
              <td>{sessions[0].training_load}</td>
              <td>{sessions[1].training_load}</td>
            </tr>
            <tr
              style={{
                textAlign: "center",
                paddingTop: "20px",
                fontWeight: "bolder",
              }}
            >
              <td style={{ borderBottom: "1px solid white" }}>
                <br />
                {translate("exercise")}
              </td>
              <td style={{ borderBottom: "1px solid white" }} colSpan={2}>
                <br />
                {translate("series")}
              </td>
            </tr>
            {exercises.map((exercise) => (
              <tr
                key={`exercise-${exercise}`}

                style={{
                  textAlign: "center",
                  borderBottom: "1px solid gray",
                }}
              >
                <td className="divider-bottom-white">
                  {translate("exercise_" + exercise.replace("-", "_"))}
                </td>
                {sessions.map((sd, idx) => (
                  <td key={`session-${idx}`} className="divider-bottom-white">
                    {sd.grouped_series[exercise]?.map((set, idx2) => (
                      <>
                        <span key={`session-${idx}-set-${exercise}-${idx2}`}>
                          {set.reps +
                            " x " +
                            formatNumber(fromKg(set.weight), 1) +
                            " " +
                            getWeightUnit()}
                        </span>
                        <br />
                      </>
                    ))}
                  </td>
                ))}

                <td></td>
              </tr>
            ))}
          </tbody>
        </table>
      </DialogContent>
    </Dialog>
  );
}
