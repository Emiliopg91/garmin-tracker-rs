import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { WorkoutListItem } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { useContext } from "react";

type Props = {
  workout: WorkoutListItem;
  onSelect: (name: string) => void;
};

export function WorkoutRow({ workout, onSelect }: Props) {
  const { translate } = useContext(I18nSettingsContext);

  return (
    <tr
      className={
        "clickable-row" + (workout.enabled ? "" : " workout-disabled")
      }
      onClick={() => onSelect(workout.name)}
    >
      <td className="text-left">
        {workout.name.length > 0 && <span>{workout.name}</span>}
        {workout.name.length == 0 && <span>{translate("other")}</span>}
      </td>
      <td>{TimeUtils.formatTimeDate(workout.latest_session)}</td>
      <td>{workout.sessions}</td>
      <td>{TimeUtils.formatDuration(workout.avg_time)}</td>
    </tr>
  );
}
