import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { ExerciseDetails } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";
import { useContext } from "react";
import { Dialog, DialogContent, DialogTitle, IconButton } from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { ExerciseVolumeChart } from "./helpers/ExerciseVolumeChart";

type Props = {
  exercise: ExerciseDetails;
  onClose: () => void;
};

export function ExerciseModal({ exercise, onClose }: Props) {
  const { translate, settings } = useContext(I18nSettingsContext);

  return (
    <Dialog open={true} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>
        {translate("exercise_" + exercise.category + "_" + exercise.id)}
        <IconButton onClick={onClose} className="modal-close-button">
          <CloseIcon />
        </IconButton>
      </DialogTitle>

      <DialogContent dividers>
        <table id="workout-details-table">
          <colgroup>
            <col className="col-200" />
            <col className="col-150" />
            <col />
          </colgroup>
          <tbody>
            <tr>
              <td>{translate("category")}:</td>
              <td>{translate("exercise_" + exercise.category)}</td>
            </tr>
            <tr>
              <td>{translate("rm")}:</td>
              <td>
                {UnitUtils.fromKg(exercise.e1rm, settings.weight_unit) +
                  " " +
                  UnitUtils.getUnit(settings.weight_unit)}
              </td>
            </tr>
            <tr>
              <td>{translate("personal_record")}:</td>
              <td>
                {exercise.reps +
                  "x" +
                  UnitUtils.fromKg(
                    exercise.weight,
                    settings.weight_unit,
                  ).toFixed(1) +
                  " " +
                  UnitUtils.getUnit(settings.weight_unit)}
              </td>
            </tr>
            <tr>
              <td>{translate("record_date")}:</td>
              <td>{TimeUtils.formatTimeDate(exercise.pr_date)}</td>
            </tr>
          </tbody>
        </table>
        {Object.keys(exercise.workouts).length > 1 && (
          <>
            <hr />
            <ExerciseVolumeChart series={exercise.series} />
            <br />
          </>
        )}
        <hr />
        <table className="full-width">
          <colgroup>
            <col className="col-350" />
            <col className="col-150" />
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
                      className="divider-bottom"
                      rowSpan={exercise.series[workout].length}
                    >
                      {workout.split("\n")[0] +
                        " @ " +
                        TimeUtils.formatTimeDate(
                          parseInt(workout.split("\n")[1]),
                        )}
                    </td>
                  )}

                  <td
                    className={`text-center${
                      idx === exercise.series[workout].length - 1
                        ? " divider-bottom"
                        : ""
                    }`}
                  >
                    {serie.reps}x
                    {UnitUtils.fromKg(
                      serie.weight,
                      settings.weight_unit,
                    ).toFixed(1)}{" "}
                    {UnitUtils.getUnit(settings.weight_unit)}
                  </td>
                </tr>
              )),
            )}
          </tbody>
        </table>
      </DialogContent>
    </Dialog>
  );
}
