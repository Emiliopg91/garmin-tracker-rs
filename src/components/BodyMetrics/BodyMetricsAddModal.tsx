import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { BodyMetricListItem } from "@/utils/backend/models";
import { UnitUtils } from "@/utils/UnitUtils";
import { useContext, useState } from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  IconButton,
  TextField,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import { DatePicker } from "@mui/x-date-pickers/DatePicker";

type Props = {
  latest: BodyMetricListItem | undefined;
  onClose: () => void;
};

type BodyMetricsListItemForm = Omit<
  BodyMetricListItem,
  "date" | "weight" | "fat_ratio" | "lean_mass" | "water_ratio"
> & {
  date: Date;
  weight: string;
  fat_ratio: string;
  lean_mass: string;
  water_ratio: string;
};

export function BodyMetricsAddModal({ latest, onClose }: Props) {
  const { translate, settings } = useContext(AppContext);
  const [data, setData] = useState<BodyMetricsListItemForm>(
    latest
      ? {
          date: new Date(),
          weight: String(latest.weight),
          fat_ratio: String(latest.fat_ratio),
          lean_mass: String(latest.lean_mass),
          water_ratio: String(latest.water_ratio),
        }
      : {
          date: new Date(),
          fat_ratio: "0",
          lean_mass: "0",
          water_ratio: "0",
          weight: "0",
        },
  );

  const onPropChange = <K extends keyof BodyMetricsListItemForm>(
    e: string | Date,
    prop: K,
  ) => {
    if (prop != "date") {
      if (typeof e !== "string") return;
      if (!/^[0-9]*[,.]?[0-9]*$/.test(e)) return;

      const str = e.replace(",", ".");
      const normalized = parseFloat(e.replace(",", "."));
      if (!isNaN(normalized)) {
        setData((prev) => ({ ...prev, [prop]: str }));
      }
    } else {
      setData((prev) => ({
        ...prev,
        date: e instanceof Date ? e : prev["date"],
      }));
    }
  };

  const onSave = () => {
    BackendClient.addBodyMeasures({
      date: Math.floor(data.date.getTime() / 1000),
      weight: UnitUtils.toKg(parseFloat(data.weight), settings.weight_unit),
      fat_ratio: parseFloat(data.fat_ratio),
      lean_mass: UnitUtils.toKg(
        parseFloat(data.lean_mass),
        settings.weight_unit,
      ),
      water_ratio: parseFloat(data.water_ratio),
    }).then(() => {
      onClose();
    });
  };

  return (
    <Dialog open={true} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>
        {translate("add_entry")}
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
            <col style={{ alignContent: "right", width: "150px" }} />
            <col />
          </colgroup>
          <tbody>
            <tr>
              <td>{translate("date")}:</td>
              <td>
                <DatePicker
                  value={data.date}
                  onChange={(value) => {
                    if (value != null) {
                      onPropChange(value, "date");
                    }
                  }}
                  format="dd/MM/yyyy"
                  slotProps={{ textField: { size: "small" } }}
                />
              </td>
            </tr>
            <tr>
              <td>{translate("weight")}:</td>
              <td>
                <TextField
                  size="small"
                  fullWidth
                  value={data.weight}
                  slotProps={{ htmlInput: { inputMode: "decimal" } }}
                  onChange={(e) => {
                    onPropChange(e.target.value, "weight");
                  }}
                />
              </td>
            </tr>
            <tr>
              <td>{translate("fat_ratio")}:</td>
              <td>
                <TextField
                  size="small"
                  fullWidth
                  value={data.fat_ratio}
                  onChange={(e) => {
                    onPropChange(e.target.value, "fat_ratio");
                  }}
                />
              </td>
            </tr>
            <tr>
              <td>{translate("lean_mass")}:</td>
              <td>
                <TextField
                  size="small"
                  fullWidth
                  value={data.lean_mass}
                  onChange={(e) => {
                    onPropChange(e.target.value, "lean_mass");
                  }}
                />
              </td>
            </tr>
            <tr>
              <td>{translate("water_ratio")}:</td>
              <td>
                <TextField
                  size="small"
                  fullWidth
                  value={data.water_ratio}
                  onChange={(e) => {
                    onPropChange(e.target.value, "water_ratio");
                  }}
                />
              </td>
            </tr>
          </tbody>
        </table>
        <hr />
        <div>
          <Button
            id="save-measure-button"
            variant="contained"
            style={{ width: "100%" }}
            onClick={onSave}
          >
            {translate("save")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
