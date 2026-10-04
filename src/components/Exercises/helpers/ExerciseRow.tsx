import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { ExerciseListItem } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { useContext } from "react";

type Props = {
  exercise: ExerciseListItem;
  onSelect: (category: number, id: number) => void;
};

export function ExerciseRow({ exercise, onSelect }: Props) {
  const { translate, fromKg, getWeightUnit } = useContext(I18nSettingsContext);

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
          fromKg(exercise.weight).toFixed(1) +
          " " +
          getWeightUnit()}
      </td>
      <td>{fromKg(exercise.e1rm) + " " + getWeightUnit()}</td>
      <td>{TimeUtils.formatDate(exercise.date)}</td>
    </tr>
  );
}
