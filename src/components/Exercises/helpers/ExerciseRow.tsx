import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { ExerciseListItem } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";
import { useContext } from "react";

type Props = {
  exercise: ExerciseListItem;
  onSelect: (category: number, id: number) => void;
};

export function ExerciseRow({ exercise, onSelect }: Props) {
  const { translate, settings } = useContext(I18nSettingsContext);

  return (
    <tr
      className="clickable-row"
      onClick={() => onSelect(exercise.category, exercise.id)}
    >
      <td className="text-left">
        {translate("exercise_" + exercise.category + "_" + exercise.id)}
      </td>
      <td>
        {exercise.reps +
          "x" +
          UnitUtils.fromKg(exercise.weight, settings.weight_unit).toFixed(1) +
          " " +
          UnitUtils.getUnit(settings.weight_unit)}
      </td>
      <td>
        {UnitUtils.fromKg(exercise.e1rm, settings.weight_unit) +
          " " +
          UnitUtils.getUnit(settings.weight_unit)}
      </td>
      <td>{TimeUtils.formatDate(exercise.date)}</td>
    </tr>
  );
}
