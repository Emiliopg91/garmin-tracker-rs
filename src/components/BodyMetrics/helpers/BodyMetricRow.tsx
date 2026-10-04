import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { BodyMetricListItem } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { useContext } from "react";

type Props = {
  measure: BodyMetricListItem;
  selectedForCompare: boolean;
  onSelect: (measure: BodyMetricListItem) => void;
  onToggleCompare: (measure: BodyMetricListItem) => void;
};

export function BodyMetricRow({
  measure,
  selectedForCompare,
  onSelect,
  onToggleCompare,
}: Props) {
  const { fromKg, getWeightUnit } = useContext(I18nSettingsContext);

  return (
    <tr
      onClick={() => onSelect(measure)}
      onContextMenu={(e) => {
        e.preventDefault();
        onToggleCompare(measure);
      }}
      className={`clickable-row ${selectedForCompare ? "row-compare-selected" : ""}`}
    >
      <td>{TimeUtils.formatDate(measure.date)}</td>
      <td>
        {fromKg(measure.weight).toFixed(1)} {getWeightUnit()}
      </td>
      <td>{measure.fat_ratio.toFixed(1)}%</td>
      <td>
        {fromKg(measure.lean_mass).toFixed(1)} {getWeightUnit()}
      </td>
      <td>{measure.water_ratio.toFixed(1)}%</td>
    </tr>
  );
}
