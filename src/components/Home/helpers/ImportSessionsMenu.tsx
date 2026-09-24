import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { LoadingContext } from "@/context/LoadingContext";
import { BackendClient } from "@/utils/backend/client";
import { DeviceListItem } from "@/utils/backend/models";
import { useContext, useState } from "react";
import { Button, Menu, MenuItem } from "@mui/material";

type Props = {
  availableDevices: DeviceListItem[];
  onImported: () => void;
};

export function ImportSessionsMenu({ availableDevices, onImported }: Props) {
  const { startLoading, finishLoading } = useContext(LoadingContext);
  const { translate } = useContext(I18nSettingsContext);
  const [importMenuAnchor, setImportMenuAnchor] = useState<{
    top: number;
    left: number;
  } | null>(null);

  const importFromDisk = () => {
    startLoading();
    BackendClient.importFromFiles()
      .then((count) => {
        if (count > 0) {
          onImported();
        }
      })
      .finally(() => {
        finishLoading();
      });
  };

  const importDevice = (serial: string) => {
    startLoading();
    BackendClient.importFromDevice(serial)
      .then((count) => {
        if (count > 0) {
          onImported();
        }
      })
      .finally(() => {
        finishLoading();
      });
  };

  return (
    <div className="list-action-bar">
      {availableDevices.length == 0 && (
        <Button
          id="import-file-toggle"
          variant="contained"
          className="full-width-button"
          onClick={importFromDisk}
        >
          {translate("import_from_disk")}
        </Button>
      )}
      {availableDevices.length > 0 && (
        <>
          <Button
            id="import-file-toggle"
            variant="contained"
            className="full-width-button"
            onClick={(e) =>
              setImportMenuAnchor({ top: e.clientY, left: e.clientX })
            }
          >
            {translate("import_sessions")}
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
            <MenuItem
              onClick={() => {
                setImportMenuAnchor(null);
                importFromDisk();
              }}
            >
              {translate("import_from_disk")}
            </MenuItem>
            {availableDevices.map((device, idx) => (
              <MenuItem
                key={"dev-" + idx}
                onClick={() => {
                  setImportMenuAnchor(null);
                  importDevice(device.serial_number);
                }}
              >
                {translate("import_from_device", [
                  device.manufacturer + " " + device.model,
                ])}
              </MenuItem>
            ))}
          </Menu>
        </>
      )}
    </div>
  );
}
