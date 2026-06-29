import { UserListItem } from "@/utils/backend/models";
import { Modal } from "react-bootstrap";

type Props = {
  measures: UserListItem;
  onClose: () => void;
};

export function UserDetailsModal({ measures, onClose }: Props) {
  return (
    <div
      className="modal show"
      style={{ display: "block", position: "initial" }}
    >
      <Modal show={true} onHide={onClose} data-bs-theme="dark">
        <Modal.Header closeButton>
          <Modal.Title>{measures.date}</Modal.Title>
        </Modal.Header>

        <Modal.Body>
          <table id="workout-details-table">
            <colgroup>
              <col style={{ width: "200px" }} />
              <col style={{ width: "150px" }} />
              <col />
            </colgroup>
            <tbody>
              <tr>
                <td>Weight:</td>
                <td>{measures.weight} Kg</td>
              </tr>
              <tr>
                <td>Fat ratio:</td>
                <td>{measures.fat_ratio}%</td>
              </tr>
              <tr>
                <td>Fat mass:</td>
                <td>
                  {(measures.weight * (measures.fat_ratio / 100)).toFixed(1)} Kg
                </td>
              </tr>
              <tr>
                <td>Lean mass:</td>
                <td>{measures.lean_mass} Kg</td>
              </tr>
              <tr>
                <td>Water ratio:</td>
                <td>{measures.water_ratio}%</td>
              </tr>
              <tr>
                <td>Water mass:</td>
                <td>
                  {(measures.weight * (measures.water_ratio / 100)).toFixed(1)}{" "}
                  Kg
                </td>
              </tr>
            </tbody>
          </table>
        </Modal.Body>
      </Modal>
    </div>
  );
}
