import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { BodyMetricListItem } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";
import { useContext } from "react";
import { Checkbox } from "@mui/material";

type Props = {
  measure: BodyMetricListItem;
  showCompare: boolean;
  compareDisabled: boolean;
  onSelect: (measure: BodyMetricListItem) => void;
  onToggleCompare: (
    event: React.MouseEvent<HTMLButtonElement>,
    measure: BodyMetricListItem,
  ) => void;
};

export function BodyMetricRow({
  measure,
  showCompare,
  compareDisabled,
  onSelect,
  onToggleCompare,
}: Props) {
  const { settings } = useContext(I18nSettingsContext);

  return (
    <tr className="clickable-row" onClick={() => onSelect(measure)}>
      <td>
        {showCompare && (
          <Checkbox
            onClick={(event) => onToggleCompare(event, measure)}
            className="compare-checkbox"
            disabled={compareDisabled}
          />
        )}
        {TimeUtils.formatDate(measure.date)}
      </td>
      <td>
        {UnitUtils.fromKg(measure.weight, settings.weight_unit).toFixed(1)}{" "}
        {UnitUtils.getUnit(settings.weight_unit)}
      </td>
      <td>{measure.fat_ratio.toFixed(1)}%</td>
      <td>
        {UnitUtils.fromKg(measure.lean_mass, settings.weight_unit).toFixed(1)}{" "}
        {UnitUtils.getUnit(settings.weight_unit)}
      </td>
      <td>{measure.water_ratio.toFixed(1)}%</td>
    </tr>
  );
}
