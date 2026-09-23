import { AppContext } from "@/context/AppContext";
import { LoadingContext } from "@/context/LoadingContext";
import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { BackendClient } from "@/utils/backend/client";
import {
  AppEnvironment,
  CloudProvider,
  DistanceUnit,
  Languages,
  WeightUnit,
} from "@/utils/backend/models";
import { useContext, useState } from "react";
import {
  Button,
  MenuItem,
  Select,
  Dialog,
  DialogContent,
  DialogTitle,
  IconButton,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";

type Props = {
  onClose: () => void;
};

export function Settings({ onClose }: Props) {
  const { environment } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { settings, translate, refreshTranslations } =
    useContext(I18nSettingsContext);

  const [weightUnit, setWeightUnit] = useState(settings.weight_unit);
  const [distanceUnit, setDistanceUnit] = useState(settings.distance_unit);
  const [autoSync, setAutoSync] = useState(settings.auto_sync);
  const [startOnBoot, setStartOnBoot] = useState(settings.start_boot);
  const [language, setLanguage] = useState(settings.language);
  const [onDeviceCOnnect, setOnDeviceConnect] = useState(
    settings.on_device_connect,
  );

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
        settings.start_boot = value;
      })
      .finally(() => {
        finishLoading();
      });
  };

  const updateLanguage = (value: Languages) => {
    startLoading();
    BackendClient.updateSettingsValue("language", value)
      .then(() => {
        setLanguage(value);
        settings.language = value;
        refreshTranslations();
      })
      .finally(() => {
        finishLoading();
      });
  };

  const updateOnDeviceConnect = (value: boolean) => {
    startLoading();
    BackendClient.updateSettingsValue(
      "on_device_connect",
      value ? "true" : "false",
    )
      .then(() => {
        setOnDeviceConnect(value);
        settings.on_device_connect = value;
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

  const uploadToCloud = (provider: CloudProvider) => {
    startLoading();
    BackendClient.uploadToCloud(provider).finally(() => {
      finishLoading();
    });
  };

  return (
    <>
      <Dialog open={true} onClose={onClose}>
        <DialogTitle>
          {translate("settings")}
          <IconButton onClick={onClose} className="modal-close-button">
            <CloseIcon />
          </IconButton>
        </DialogTitle>

        <DialogContent dividers>
          <fieldset>
            <legend>{translate("user_settings")}</legend>
            <table>
              <colgroup>
                <col className="col-250"></col>
                <col className="col-200"></col>
              </colgroup>
              <tbody>
                <tr>
                  <td>{translate("language")}</td>
                  <td>
                    <Select
                      size="small"
                      fullWidth
                      value={language}
                      onChange={(e) =>
                        updateLanguage(e.target.value as Languages)
                      }
                    >
                      <MenuItem value={Languages.English}>
                        {translate("lang_en")}
                      </MenuItem>
                      <MenuItem value={Languages.Spanish}>
                        {translate("lang_es")}
                      </MenuItem>
                    </Select>
                  </td>
                </tr>
                <tr>
                  <td>{translate("weight_unit")}</td>
                  <td>
                    <Select
                      size="small"
                      fullWidth
                      value={weightUnit}
                      onChange={(e) =>
                        updateWeightUnit(e.target.value as WeightUnit)
                      }
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
              </tbody>
            </table>
          </fieldset>
          <br />
          <fieldset>
            <legend>{translate("application")}</legend>
            <table>
              <colgroup>
                <col className="col-250"></col>
                <col className="col-200"></col>
              </colgroup>
              <tbody>
                <tr>
                  <td>{translate("start_on_boot")}</td>
                  <td>
                    <Select
                      size="small"
                      fullWidth
                      value={startOnBoot ? "true" : "false"}
                      onChange={(e) =>
                        updateStartOnBoot(e.target.value === "true")
                      }
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
                  <td>{translate("on_device_connect")}</td>
                  <td>
                    <Select
                      size="small"
                      fullWidth
                      value={onDeviceCOnnect ? "true" : "false"}
                      onChange={(e) =>
                        updateOnDeviceConnect(e.target.value === "true")
                      }
                    >
                      <MenuItem value="false">
                        {translate("do_nothing")}
                      </MenuItem>
                      <MenuItem value="true">
                        {translate("start_automatically")}
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
                      onChange={(e) =>
                        updateAutoSync(e.target.value === "true")
                      }
                    >
                      <MenuItem value="true">
                        {translate("auto_sync_true")}
                      </MenuItem>
                      <MenuItem value="false">
                        {translate("auto_sync_false")}
                      </MenuItem>
                    </Select>
                  </td>
                </tr>
                <tr>
                  <td>{translate("database_operations")}</td>
                  <td>
                    <table style={{ width: "100%" }}>
                      <tbody>
                        <tr>
                          <td>
                            <Button
                              id="add-measure-button"
                              variant="contained"
                              className="full-width-button"
                              onClick={exportDatabase}
                            >
                              {translate("backup_database")}
                            </Button>
                          </td>
                        </tr>
                        <tr>
                          <td>
                            <Button
                              id="upload-onedrive-button"
                              variant="contained"
                              className="full-width-button"
                              onClick={()=>{uploadToCloud(CloudProvider.OneDrive)}}
                            >
                              {translate("upload_onedrive")}
                            </Button>
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </td>
                </tr>
              </tbody>
            </table>
          </fieldset>
        </DialogContent>
      </Dialog>
    </>
  );
}
