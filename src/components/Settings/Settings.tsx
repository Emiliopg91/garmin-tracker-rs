import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import {
  AppEnvironment,
  DistanceUnit,
  WeightUnit,
} from "@/utils/backend/models";
import { useContext, useState } from "react";
import { Form } from "react-bootstrap";

export function Settings() {
  const { environment, settings, translate } = useContext(AppContext);
  console.log(environment);

  const [weightUnit, setWeightUnit] = useState(settings.weight_unit);
  const [distanceUnit, setDistanceUnit] = useState(settings.distance_unit);
  const [autoSync, setAutoSync] = useState(settings.auto_sync);
  const [startOnBoot, setStartOnBoot] = useState(settings.start_boot);

  const updateWeightUnit = (value: WeightUnit) => {
    BackendClient.updateSettingsValue("weight_unit", value).then(() => {
      setWeightUnit(value);
      settings.weight_unit = value;
    });
  };

  const updateDistanceUnit = (value: DistanceUnit) => {
    BackendClient.updateSettingsValue("distance_unit", value).then(() => {
      setDistanceUnit(value);
      settings.distance_unit = value;
    });
  };

  const updateAutoSync = (value: boolean) => {
    BackendClient.updateSettingsValue(
      "auto_sync",
      value ? "true" : "false",
    ).then(() => {
      setAutoSync(value);
      settings.auto_sync = value;
    });
  };

  const updateStartOnBoot = (value: boolean) => {
    BackendClient.updateSettingsValue(
      "start_boot",
      value ? "true" : "false",
    ).then(() => {
      setStartOnBoot(value);
      settings.auto_sync = value;
    });
  };

  return (
    <>
      <table>
        <tr>
          <td>{translate("weight_unit")}</td>
          <td>
            <Form.Select
              value={weightUnit}
              onChange={(e) => updateWeightUnit(e.target.value as WeightUnit)}
            >
              <option value={WeightUnit.Kilograms}>
                {translate("weight_unit_kilograms")}
              </option>
              <option value={WeightUnit.Pounds}>
                {translate("weight_unit_pounds")}
              </option>
            </Form.Select>
          </td>
        </tr>
        <tr>
          <td>{translate("distance_unit")}</td>
          <td>
            <Form.Select
              value={distanceUnit}
              onChange={(e) =>
                updateDistanceUnit(e.target.value as DistanceUnit)
              }
            >
              <option value={DistanceUnit.Kilometers}>
                {translate("distance_unit_kilometers")}
              </option>
              <option value={DistanceUnit.Miles}>
                {translate("distance_unit_miles")}
              </option>
            </Form.Select>
          </td>
        </tr>
        <tr>
          <td>{translate("auto_sync")}</td>
          <td>
            <Form.Select
              value={autoSync ? "true" : "false"}
              onChange={(e) => updateAutoSync(e.target.value === "true")}
            >
              <option value="true">{translate("auto_sync_true")}</option>
              <option value="false">{translate("auto_sync_false")}</option>
            </Form.Select>
          </td>
        </tr>
        <tr>
          <td>{translate("start_on_boot")}</td>
          <td>
            <Form.Select
              value={startOnBoot ? "true" : "false"}
              onChange={(e) => updateStartOnBoot(e.target.value === "true")}
              disabled={environment == AppEnvironment.Debug}
            >
              <option value="false">{translate("start_on_boot_false")}</option>
              <option value="true">{translate("start_on_boot_true")}</option>
            </Form.Select>
          </td>
        </tr>
      </table>
    </>
  );
}
