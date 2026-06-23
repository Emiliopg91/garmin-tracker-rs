import { AppContext } from "@/context/AppContext";
import { BackendClient } from "@/utils/backend/client";
import { SessionDetails, SessionListItem } from "@/utils/backend/models";
import { useContext, useEffect, useState } from "react";
import { Dropdown } from "react-bootstrap";
import { SessionModal } from "./SessionModal";

export function SessionsList() {
  const { setLoading, availableDevices } = useContext(AppContext);

  const [sessions, setSessions] = useState<SessionListItem[]>([]);
  const [sessionDetails, setSessionDetails] = useState<
    SessionDetails | undefined
  >(undefined);

  const refreshList = () => {
    setLoading(true);
    BackendClient.getSessions()
      .then((data) => {
        setSessions(data);
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error getting session list",
          body: e,
        });
      })
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    refreshList();
  }, []);

  const importFile = () => {
    setLoading(true);
    BackendClient.importFromFile()
      .then((count) => {
        BackendClient.showNotification({
          title: "File imported succesfully",
          body: "Imported " + count + " sessions from file",
        });
        refreshList();
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error on file import",
          body: e,
        });
      })
      .finally(() => {
        setLoading(false);
      });
  };

  const importDevice = (serial: string) => {
    setLoading(true);
    BackendClient.importFromDevice(serial)
      .then((count) => {
        refreshList();
        BackendClient.showNotification({
          title: "Imported succesfully from device",
          body: "Imported " + count + " sessions from device",
        });
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error on file import",
          body: e,
        });
      })
      .finally(() => {
        setLoading(false);
      });
  };

  const getSessionDetails = (timestamp: string) => {
    setLoading(true);
    BackendClient.getSessionDetails(timestamp)
      .then((details) => {
        setSessionDetails(details);
      })
      .catch((e) => {
        BackendClient.showNotification({
          title: "Error getting session details",
          body: e,
        });
      })
      .finally(() => {
        setLoading(false);
      });
  };

  return (
    <>
      <div id="list-layer">
        <table>
          <thead>
            <tr>
              <th style={{ textAlign: "center" }}>Workout</th>
              <th style={{ textAlign: "center" }}>Date</th>
              <th style={{ textAlign: "center" }}>Exercises</th>
              <th style={{ textAlign: "center" }}>Sets</th>
              <th style={{ textAlign: "center" }}>Volume</th>
            </tr>
          </thead>

          <tbody>
            {sessions.map((session, idx) => (
              <tr
                key={idx}
                onClick={() => getSessionDetails(session.timestamp)}
                style={{ cursor: "pointer" }}
              >
                <td>{session.name}</td>
                <td>{session.date}</td>
                <td>{session.exercises_num}</td>
                <td>{session.series_num}</td>
                <td>{session.volume} Kg</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div>
        {sessionDetails && (
          <SessionModal
            session={sessionDetails}
            onClose={() => setSessionDetails(undefined)}
            onUpdate={() => refreshList()}
          />
        )}
      </div>
      <div style={{ padding: "5px", width: "100%", marginTop: "auto" }}>
        <Dropdown id="import-file-dropdown" className="w-100">
          <Dropdown.Toggle id="import-file-toggle">
            Import sessions
          </Dropdown.Toggle>

          <Dropdown.Menu id="import-file-menu">
            <Dropdown.Item key={"file"} onClick={importFile}>
              From file
            </Dropdown.Item>
            {availableDevices.length > 0 &&
              availableDevices.map((device, idx) => (
                <Dropdown.Item
                  key={"dev-" + idx}
                  onClick={() => {
                    importDevice(device.serial_number);
                  }}
                >
                  From {device.manufacturer + " " + device.model}
                </Dropdown.Item>
              ))}
            {availableDevices.length == 0 && (
              <Dropdown.Item disabled={true}>No device found</Dropdown.Item>
            )}
          </Dropdown.Menu>
        </Dropdown>
      </div>
    </>
  );
}
