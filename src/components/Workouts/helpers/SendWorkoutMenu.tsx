import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { LoadingContext } from "@/context/LoadingContext";
import { BackendClient } from "@/utils/backend/client";
import { useContext, useState } from "react";
import { Button, Menu, MenuItem } from "@mui/material";
import { AppContext } from "@/context/AppContext";
import { WorkoutDetails } from "@/utils/backend/models";

type Props = {
  workout: WorkoutDetails;
};

export function SendWorkoutMenu({ workout }: Props) {
  const { availableDevices } = useContext(AppContext);
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate } = useContext(I18nSettingsContext);
  const [importMenuAnchor, setImportMenuAnchor] = useState<{
    top: number;
    left: number;
  } | null>(null);

  const importDevice = (serial: string) => {
    startLoading();
    BackendClient.sendToDevice(workout.name, serial).finally(() => {
      finishLoading();
    });
  };

  return (
    <div className="list-action-bar">
      <>
        <Button
          id="send-workout"
          variant="contained"
          className="full-width-button"
          onClick={(e) =>
            setImportMenuAnchor({ top: e.clientY, left: e.clientX })
          }
          disabled={availableDevices.length == 0}
        >
          {translate("send_to")}
        </Button>
        <Menu
          id="import-file-menu"
          anchorReference="anchorPosition"
          anchorPosition={importMenuAnchor ?? undefined}
          open={Boolean(importMenuAnchor)}
          onClose={() => setImportMenuAnchor(null)}
          anchorOrigin={{ vertical: "top", horizontal: "left" }}
          transformOrigin={{ vertical: "bottom", horizontal: "left" }}
        >
          {availableDevices.map((device, idx) => (
            <MenuItem
              key={"dev-" + idx}
              onClick={() => {
                setImportMenuAnchor(null);
                importDevice(device.serial_number);
              }}
            >
              {device.manufacturer + " " + device.model}
            </MenuItem>
          ))}
        </Menu>
      </>
    </div>
  );
}
