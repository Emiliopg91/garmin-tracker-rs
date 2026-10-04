import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { WorkoutListItem } from "@/utils/backend/models";
import { useContext } from "react";

type Props = {
  workout: WorkoutListItem;
  onSelect: (name: string) => void;
};

export function WorkoutRow({ workout, onSelect }: Props) {
  const { translate, formatDuration, formatTimeDate, formatNumber } =
    useContext(I18nSettingsContext);

  return (
    <tr
      className={"clickable-row" + (workout.enabled ? "" : " workout-disabled")}
      onClick={() => onSelect(workout.name)}
    >
      <td className="text-left">
        {workout.name.length > 0 && <span>{workout.name}</span>}
        {workout.name.length == 0 && <span>{translate("other")}</span>}
      </td>
      <td>
        {workout.latest_session && formatTimeDate(workout.latest_session)}
      </td>
      <td>{formatNumber(workout.sessions, 0)}</td>
      <td>{workout.avg_time > 0 && formatDuration(workout.avg_time)}</td>
    </tr>
  );
}
