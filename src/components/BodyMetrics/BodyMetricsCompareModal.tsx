import { AppContext } from "@/context/AppContext";
import { BodyMetricListItem } from "@/utils/backend/models";
import { UnitUtils } from "@/utils/UnitUtils";
import { useContext } from "react";
import { Dialog, DialogContent, DialogTitle, IconButton } from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { TimeUtils } from "@/utils/TimeUtils";

type Props = {
  measures: BodyMetricListItem[];
  onClose: () => void;
};

export function BodyMetricsCompareModal({ measures, onClose }: Props) {
  const { translate, settings } = useContext(AppContext);

  return (
    <Dialog open={true} onClose={onClose}>
      <DialogTitle>
        {translate("metrics_comparaison")}
        <IconButton
          onClick={onClose}
          sx={{ position: "absolute", right: 8, top: 8 }}
        >
          <CloseIcon />
        </IconButton>
      </DialogTitle>

      <DialogContent dividers>
        <table id="workout-details-table">
          <colgroup>
            <col style={{ width: "200px" }} />
            <col style={{ width: "150px" }} />
            <col style={{ width: "150px" }} />
            <col />
          </colgroup>
          <thead>
            <tr>
              <td style={{ borderBottom: "1px solid white" }}></td>
              {measures.map((entry, idx) => (
                <td
                  key={"entry-" + idx}
                  style={{ borderBottom: "1px solid white" }}
                >
                  <b>{TimeUtils.formatDate(entry.date)}</b>
                </td>
              ))}
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>{translate("weight")}:</td>
              {measures.map((entry, idx) => (
                <td key={"entry-" + idx}>
                  {UnitUtils.fromKg(entry.weight, settings.weight_unit).toFixed(
                    1,
                  )}{" "}
                  {UnitUtils.getUnit(settings.weight_unit)}
                </td>
              ))}
            </tr>
            <tr>
              <td>{translate("fat_ratio")}:</td>
              {measures.map((entry, idx) => (
                <td key={"entry-" + idx}>{entry.fat_ratio}%</td>
              ))}
            </tr>
            <tr>
              <td>{translate("fat_mass")}:</td>
              {measures.map((entry, idx) => (
                <td key={"entry-" + idx}>
                  {UnitUtils.fromKg(
                    entry.weight * (entry.fat_ratio / 100),
                    settings.weight_unit,
                  ).toFixed(1)}{" "}
                  {UnitUtils.getUnit(settings.weight_unit)}
                </td>
              ))}
            </tr>
            <tr>
              <td>{translate("lean_mass")}:</td>
              {measures.map((entry, idx) => (
                <td key={"entry-" + idx}>
                  {UnitUtils.fromKg(
                    entry.lean_mass,
                    settings.weight_unit,
                  ).toFixed(1)}{" "}
                  {UnitUtils.getUnit(settings.weight_unit)}
                </td>
              ))}
            </tr>
            <tr>
              <td>{translate("water_ratio")}:</td>
              {measures.map((entry, idx) => (
                <td key={"entry-" + idx}>{entry.water_ratio}%</td>
              ))}
            </tr>
            <tr>
              <td>{translate("water_mass")}:</td>
              {measures.map((entry, idx) => (
                <td key={"entry-" + idx}>
                  {UnitUtils.fromKg(
                    entry.weight * (entry.water_ratio / 100),
                    settings.weight_unit,
                  ).toFixed(1)}{" "}
                  {UnitUtils.getUnit(settings.weight_unit)}
                </td>
              ))}
            </tr>
          </tbody>
        </table>
      </DialogContent>
    </Dialog>
  );
}
