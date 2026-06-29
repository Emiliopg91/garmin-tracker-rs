import { BackendClient } from "@/utils/backend/client";
import { UserListItem } from "@/utils/backend/models";
import { useState } from "react";
import { Button, Modal } from "react-bootstrap";

type Props = {
  latest: UserListItem | undefined;
  onClose: () => void;
};

export function UserAddModal({ latest, onClose }: Props) {
  const [data, setData] = useState<UserListItem>(
    latest
      ? { ...latest }
      : {
          date: "",
          fat_ratio: 0,
          lean_mass: 0,
          water_ratio: 0,
          weight: 0,
        },
  );

  const onPropChange = <K extends keyof UserListItem>(e: string, prop: K) => {
    setData((prev) => ({
      ...prev,
      [prop]: parseFloat(e.replace(",", ".")),
    }));
  };

  const onSave = () => {
    const date = new Date();
    const hours = date.getHours().toString().padStart(2, "0");
    const minutes = date.getMinutes().toString().padStart(2, "0");
    const day = date.getDate().toString().padStart(2, "0");
    const month = (date.getMonth() + 1).toString().padStart(2, "0");
    const year = date.getFullYear();

    data.date = `${hours}:${minutes} ${day}/${month}/${year}`;

    BackendClient.addUserMeasures(data).then(() => {
      onClose();
    });
  };

  return (
    <div
      className="modal show"
      style={{ display: "block", position: "initial" }}
    >
      <Modal show={true} onHide={onClose} data-bs-theme="dark">
        <Modal.Header closeButton>
          <Modal.Title>Add entry</Modal.Title>
        </Modal.Header>

        <Modal.Body>
          <table id="workout-details-table">
            <colgroup>
              <col style={{ alignContent: "right", width: "150px" }} />
              <col />
            </colgroup>
            <tbody>
              <tr>
                <td>Weight:</td>
                <td>
                  <input
                    type="number"
                    value={data.weight}
                    min={0}
                    step={0.1}
                    onChange={(e) => {
                      onPropChange(e.target.value, "weight");
                    }}
                  />
                </td>
              </tr>
              <tr>
                <td>Fat ratio:</td>
                <td>
                  <input
                    type="number"
                    value={data.fat_ratio}
                    min={0}
                    step={0.1}
                    onChange={(e) => {
                      onPropChange(e.target.value, "fat_ratio");
                    }}
                  />
                </td>
              </tr>
              <tr>
                <td>Lean mass:</td>
                <td>
                  <input
                    type="number"
                    value={data.lean_mass}
                    min={0}
                    step={0.1}
                    onChange={(e) => {
                      onPropChange(e.target.value, "lean_mass");
                    }}
                  />
                </td>
              </tr>
              <tr>
                <td>Water ratio:</td>
                <td>
                  <input
                    type="number"
                    value={data.water_ratio}
                    min={0}
                    step={0.1}
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
              style={{ width: "100%" }}
              onClick={onSave}
            >
              Save
            </Button>
          </div>
        </Modal.Body>
      </Modal>
    </div>
  );
}
