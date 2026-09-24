import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { Languages, WorkoutDetails } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";
import { useContext, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  IconButton,
  Switch,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { BackendClient } from "@/utils/backend/client";
import { LoadingContext } from "@/context/LoadingContext";
import { WorkoutVolumeChart } from "./helpers/WorkoutVolumeChart";

const NUMBER_LOCALES: Record<Languages, string> = {
  [Languages.Spanish]: "es-ES",
  [Languages.English]: "en-US",
};

type Props = {
  workout: WorkoutDetails;
  showEnable: boolean;
  onClose: () => void;
  onUpdate?: (name: string, status: boolean) => void;
};

export function WorkoutModal({
  workout,
  showEnable,
  onClose,
  onUpdate,
}: Props) {
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate, settings } = useContext(I18nSettingsContext);
  const [enabled, setEnabled] = useState(workout.enabled);

  const toggleEnabled = () => {
    startLoading();
    BackendClient.setWorkoutStatus(workout.name, !enabled)
      .then(() => {
        if (onUpdate) {
          onUpdate(workout.name, !enabled);
        }
        setEnabled((prev) => !prev);
      })
      .finally(() => {
        finishLoading();
      });
  };

  return (
    <Dialog open={true} onClose={onClose} fullWidth maxWidth="md">
      <DialogTitle>
        {workout.name.length > 0 && <span>{workout.name}</span>}
        {workout.name.length == 0 && <span>{translate("other")}</span>}
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
              <td>{translate("sessions")}:</td>
              <td>{workout.session_count}</td>
            </tr>
            <tr>
              <td>{translate("latest_session")}</td>
              <td>{TimeUtils.formatTimeDate(workout.latest_session)}</td>
            </tr>
            <tr>
              <td>{translate("average_time")}</td>
              <td>{TimeUtils.formatDuration(workout.avg_time)}</td>
            </tr>
            {workout.name.length > 0 && (
              <tr>
                <td>{translate("average_volume")}:</td>
                <td>
                  {UnitUtils.fromKg(
                    workout.avg_volume,
                    settings.weight_unit,
                  ).toFixed(1)}{" "}
                  {UnitUtils.getUnit(settings.weight_unit)}
                </td>
              </tr>
            )}
            {showEnable && (
              <tr>
                <td>{translate("enabled")}</td>
                <td>
                  <Switch
                    checked={enabled}
                    onChange={toggleEnabled}
                    slotProps={{
                      input: { "aria-label": translate("enabled") },
                    }}
                  />
                </td>
              </tr>
            )}
          </tbody>
        </table>
        {workout.sessions.length > 1 && (
          <>
            <hr />
            {workout.avg_volume > 0 && (
              <WorkoutVolumeChart sessions={workout.sessions} />
            )}
          </>
        )}
        <hr />
        <table>
          <colgroup>
            <col className={workout.avg_volume > 0 ? "col-230" : "col-370"} />
            <col className={workout.avg_volume > 0 ? "col-120" : "col-260"} />
            {workout.avg_volume > 0 && <col className="col-280" />}
          </colgroup>
          <thead>
            <tr>
              <th>{translate("date")}</th>
              <th>{translate("time")}</th>
              {workout.avg_volume > 0 && <th>{translate("volume")}</th>}
            </tr>
          </thead>
          <tbody>
            {workout.sessions.map((session, idx) => (
              <tr key={idx} className="divider-bottom">
                <td className="text-center">
                  {TimeUtils.formatTimeDate(session.date)}
                </td>
                <td className="text-center">
                  {TimeUtils.formatDuration(session.time)}
                </td>
                {workout.avg_volume > 0 && (
                  <td className="text-center">
                    {UnitUtils.fromKg(
                      session.volume,
                      settings.weight_unit,
                    ).toFixed(1)}{" "}
                    {UnitUtils.getUnit(settings.weight_unit)}
                    {idx < workout.sessions.length - 1 &&
                      " (" +
                        new Intl.NumberFormat(
                          NUMBER_LOCALES[settings.language],
                          {
                            style: "percent",
                            minimumFractionDigits: 2,
                            maximumFractionDigits: 2,
                            signDisplay: "always",
                          },
                        ).format(
                          (workout.sessions[idx].volume -
                            workout.sessions[idx + 1].volume) /
                            workout.sessions[idx + 1].volume,
                        ) +
                        ")"}
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </DialogContent>
    </Dialog>
  );
}
