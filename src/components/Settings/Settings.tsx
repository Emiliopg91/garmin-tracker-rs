import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import {
  AppEnvironment,
  DistanceUnit,
  WeightUnit,
} from "@/utils/backend/models";
import { useContext, useState } from "react";
import { Button, MenuItem, Select } from "@mui/material";

export function Settings() {
  const { environment, settings, translate, startLoading, finishLoading } =
    useContext(AppContext);
  console.log(environment);

  const [weightUnit, setWeightUnit] = useState(settings.weight_unit);
  const [distanceUnit, setDistanceUnit] = useState(settings.distance_unit);
  const [autoSync, setAutoSync] = useState(settings.auto_sync);
  const [startOnBoot, setStartOnBoot] = useState(settings.start_boot);

  const updateWeightUnit = (value: WeightUnit) => {
    startLoading();
    BackendClient.updateSettingsValue("weight_unit", value)
      .then(() => {
        setWeightUnit(value);
        settings.weight_unit = value;
      })
      .finally(() => {
        finishLoading();
      });
  };

  const updateDistanceUnit = (value: DistanceUnit) => {
    startLoading();
    BackendClient.updateSettingsValue("distance_unit", value)
      .then(() => {
        setDistanceUnit(value);
        settings.distance_unit = value;
      })
      .finally(() => {
        finishLoading();
      });
  };

  const updateAutoSync = (value: boolean) => {
    startLoading();
    BackendClient.updateSettingsValue("auto_sync", value ? "true" : "false")
      .then(() => {
        setAutoSync(value);
        settings.auto_sync = value;
      })
      .finally(() => {
        finishLoading();
      });
  };

  const updateStartOnBoot = (value: boolean) => {
    startLoading();
    BackendClient.updateSettingsValue("start_boot", value ? "true" : "false")
      .then(() => {
        setStartOnBoot(value);
        settings.auto_sync = value;
      })
      .finally(() => {
        finishLoading();
      });
  };

  const exportDatabase = () => {
    startLoading();
    BackendClient.exportDatabase().finally(() => {
      finishLoading();
    });
  };

  return (
    <>
      <table>
        <tr>
          <td>{translate("weight_unit")}</td>
          <td>
            <Select
              size="small"
              fullWidth
              value={weightUnit}
              onChange={(e) => updateWeightUnit(e.target.value as WeightUnit)}
            >
              <MenuItem value={WeightUnit.Kilograms}>
                {translate("weight_unit_kilograms")}
              </MenuItem>
              <MenuItem value={WeightUnit.Pounds}>
                {translate("weight_unit_pounds")}
              </MenuItem>
            </Select>
          </td>
        </tr>
        <tr>
          <td>{translate("distance_unit")}</td>
          <td>
            <Select
              size="small"
              fullWidth
              value={distanceUnit}
              onChange={(e) =>
                updateDistanceUnit(e.target.value as DistanceUnit)
              }
            >
              <MenuItem value={DistanceUnit.Kilometers}>
                {translate("distance_unit_kilometers")}
              </MenuItem>
              <MenuItem value={DistanceUnit.Miles}>
                {translate("distance_unit_miles")}
              </MenuItem>
            </Select>
          </td>
        </tr>
        <tr>
          <td>{translate("auto_sync")}</td>
          <td>
            <Select
              size="small"
              fullWidth
              value={autoSync ? "true" : "false"}
              onChange={(e) => updateAutoSync(e.target.value === "true")}
            >
              <MenuItem value="true">{translate("auto_sync_true")}</MenuItem>
              <MenuItem value="false">{translate("auto_sync_false")}</MenuItem>
            </Select>
          </td>
        </tr>
        <tr>
          <td>{translate("start_on_boot")}</td>
          <td>
            <Select
              size="small"
              fullWidth
              value={startOnBoot ? "true" : "false"}
              onChange={(e) => updateStartOnBoot(e.target.value === "true")}
              disabled={environment == AppEnvironment.Debug}
            >
              <MenuItem value="false">
                {translate("start_on_boot_false")}
              </MenuItem>
              <MenuItem value="true">
                {translate("start_on_boot_true")}
              </MenuItem>
            </Select>
          </td>
        </tr>
        <tr>
          <td>{translate("database_operations")}</td>
          <td>
            <Button
              id="add-measure-button"
              variant="contained"
              style={{ width: "100%" }}
              onClick={exportDatabase}
            >
              {translate("backup_database")}
            </Button>
          </td>
        </tr>
      </table>
    </>
  );
}
