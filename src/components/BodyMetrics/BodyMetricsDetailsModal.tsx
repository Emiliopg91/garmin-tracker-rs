import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { BodyMetricListItem } from "@/utils/backend/models";
import { TimeUtils } from "@/utils/TimeUtils";
import { UnitUtils } from "@/utils/UnitUtils";
import { useContext } from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  IconButton,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";

type Props = {
  measures: BodyMetricListItem;
  onClose: () => void;
  onDelete: () => void;
};

export function BodyMetricsDetailsModal({
  measures,
  onClose,
  onDelete,
}: Props) {
  const { translate, startLoading, finishLoading, settings } =
    useContext(AppContext);

  const deleteEntry = () => {
    startLoading();
    BackendClient.deleteBodyMetric(measures.date)
      .then(() => {
        onDelete();
        onClose();
      })
      .finally(() => {
        finishLoading();
      });
  };

  return (
    <Dialog open={true} onClose={onClose}>
      <DialogTitle>
        {TimeUtils.formatDate(measures.date)}
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
              <td>{translate("weight")}:</td>
              <td>
                {UnitUtils.fromKg(
                  measures.weight,
                  settings.weight_unit,
                ).toFixed(1)}{" "}
                {UnitUtils.getUnit(settings.weight_unit)}
              </td>
            </tr>
            <tr>
              <td>{translate("fat_ratio")}:</td>
              <td>{measures.fat_ratio}%</td>
            </tr>
            <tr>
              <td>{translate("fat_mass")}:</td>
              <td>
                {UnitUtils.fromKg(
                  measures.weight * (measures.fat_ratio / 100),
                  settings.weight_unit,
                ).toFixed(1)}{" "}
                {UnitUtils.getUnit(settings.weight_unit)}
              </td>
            </tr>
            <tr>
              <td>{translate("lean_mass")}:</td>
              <td>
                {UnitUtils.fromKg(
                  measures.lean_mass,
                  settings.weight_unit,
                ).toFixed(1)}{" "}
                {UnitUtils.getUnit(settings.weight_unit)}
              </td>
            </tr>
            <tr>
              <td>{translate("water_ratio")}:</td>
              <td>{measures.water_ratio}%</td>
            </tr>
            <tr>
              <td>{translate("water_mass")}:</td>
              <td>
                {UnitUtils.fromKg(
                  measures.weight * (measures.water_ratio / 100),
                  settings.weight_unit,
                ).toFixed(1)}{" "}
                {UnitUtils.getUnit(settings.weight_unit)}
              </td>
            </tr>
          </tbody>
        </table>
        <div>
          <hr />
          <Button
            id="import-button"
            className="full-width-button"
            variant="contained"
            color="error"
            onClick={deleteEntry}
          >
            {translate("delete_entry")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
