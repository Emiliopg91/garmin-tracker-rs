import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";
import { SessionFrontDetails } from "@/utils/SessionUtils";
import { useContext } from "react";

type Props = {
  session: SessionFrontDetails;
};

export function SessionDetailsTable({ session }: Props) {
  const { translate, settings } = useContext(I18nSettingsContext);

  return (
    <table id="session-details-table">
      <colgroup>
        <col className="col-200" />
        <col className="col-250" />
        <col />
      </colgroup>
      <tbody>
        <tr>
          <td>{translate("date")}:</td>
          <td>{TimeUtils.formatTimeDate(session.timestamp)}</td>
        </tr>
        <tr>
          <td>{translate("time")}:</td>
          <td>
            {(session.active_time > 0
              ? TimeUtils.formatDuration(session.active_time) + " / "
              : "") + TimeUtils.formatDuration(session.total_elapsed_time)}
          </td>
        </tr>
        <tr>
          <td>{translate("calories")}:</td>
          <td>
            {session.total_calories -
              session.metabolic_calories +
              " / " +
              session.total_calories}{" "}
            Kcal
          </td>
        </tr>
        {session.distance != null && session.distance > 0 && (
          <>
            <tr>
              <td>{translate("distance")}:</td>
              <td>
                {UnitUtils.fromKm(
                  session.distance,
                  settings.distance_unit,
                ).toFixed(2)}{" "}
                {UnitUtils.getUnit(settings.distance_unit)}
              </td>
            </tr>
            <tr>
              <td>{translate("speed")}:</td>
              <td>
                {UnitUtils.fromKm(
                  session.distance / (session.total_elapsed_time / 3600),
                  settings.distance_unit,
                ).toFixed(2)}{" "}
                {UnitUtils.getUnit(settings.distance_unit)}/h (
                {TimeUtils.formatDuration(
                  UnitUtils.toKm(
                    session.total_elapsed_time / session.distance,
                    settings.distance_unit,
                  ),
                )}{" "}
                min/{UnitUtils.getUnit(settings.distance_unit)}){" "}
              </td>
            </tr>
          </>
        )}
        {session.elevations && (
          <tr>
            <td>{translate("elevations")}:</td>
            <td>
              {Math.round(session.elevations[0]) +
                " ↑   ↓ " +
                Math.round(session.elevations[1])}
            </td>
          </tr>
        )}
        {session.volume > 0 && (
          <tr>
            <td>{translate("volume")}:</td>
            <td>
              {UnitUtils.fromKg(session.volume, settings.weight_unit).toFixed(
                1,
              )}{" "}
              {UnitUtils.getUnit(settings.weight_unit)}
            </td>
          </tr>
        )}
        <tr>
          <td>{translate("workout_load")}:</td>
          <td>{session.training_load}</td>
        </tr>
        {session.device && (
          <tr>
            <td>{translate("imported_from")}:</td>
            <td>{session.device}</td>
          </tr>
        )}
      </tbody>
    </table>
  );
}
