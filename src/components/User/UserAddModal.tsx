import { BackendClient } from "@/utils/backend/client";
import { UserListItem } from "@/utils/backend/models";
import { useState } from "react";
import { Button, Modal } from "react-bootstrap";
import DatePicker from "react-datepicker";
import "react-datepicker/dist/react-datepicker.css";

type Props = {
  latest: UserListItem | undefined;
  onClose: () => void;
};

type UserListItemForm = Omit<
  UserListItem,
  "date" | "weight" | "fat_ratio" | "lean_mass" | "water_ratio"
> & {
  date: Date;
  weight: string;
  fat_ratio: string;
  lean_mass: string;
  water_ratio: string;
};

export function UserAddModal({ latest, onClose }: Props) {
  const [data, setData] = useState<UserListItemForm>(
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

  const onPropChange = <K extends keyof UserListItemForm>(
    e: string | Date,
    prop: K,
  ) => {
    console.log(prop + " -> " + e);
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
    const dateStr = `00:00 ${String(data.date.getDate()).padStart(2, "0")}/${String(data.date.getMonth() + 1).padStart(2, "0")}/${data.date.getFullYear()}`;
    BackendClient.addUserMeasures({
      date: dateStr,
      weight: parseFloat(data.weight),
      fat_ratio: parseFloat(data.fat_ratio),
      lean_mass: parseFloat(data.lean_mass),
      water_ratio: parseFloat(data.water_ratio),
    }).then(() => {
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
                <td>Date:</td>
                <td>
                  <DatePicker
                    onChange={(value: Date | null) => {
                      if (value != null) {
                        onPropChange(value, "date");
                      }
                    }}
                    selected={data.date}
                    dateFormat="dd/MM/yyyy"
                  />
                </td>
              </tr>
              <tr>
                <td>Weight:</td>
                <td>
                  <input
                    type="text"
                    value={data.weight}
                    inputMode="decimal"
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
                    type="text"
                    value={data.fat_ratio}
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
                    type="text"
                    value={data.lean_mass}
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
                    type="text"
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
