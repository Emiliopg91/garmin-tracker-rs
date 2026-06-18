import { WorkoutDetails } from "@/utils/RustBridge";
import { Modal } from "react-bootstrap";

type Props = {
  workout: WorkoutDetails;
  onClose: () => void;
};

export function WorkoutModal({ workout, onClose }: Props) {
  return (
    <div
      className="modal show"
      style={{ display: "block", position: "initial" }}
    >
      <Modal show={true} onHide={onClose} data-bs-theme="dark">
        <Modal.Header closeButton>
          <Modal.Title>{workout.name}</Modal.Title>
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
                <td>Sessions:</td>
                <td>{workout.session_count}</td>
              </tr>
              <tr>
                <td>Latest session:</td>
                <td>{workout.latest_session}</td>
              </tr>
              <tr>
                <td>Average time:</td>
                <td>{workout.avg_time}</td>
              </tr>
              <tr>
                <td>Average volume:</td>
                <td>{workout.avg_volume} Kg</td>
              </tr>
            </tbody>
          </table>
          {workout.sessions.length > 0 && (
            <>
              <hr />
              <table>
                <colgroup>
                  <col style={{ width: "230px" }} />
                  <col style={{ width: "120px" }} />
                  <col style={{ width: "280px" }} />
                </colgroup>
                <thead>
                  <tr>
                    <th>Date</th>
                    <th>Time</th>
                    <th>Volume</th>
                  </tr>
                </thead>
                <tbody>
                  {workout.sessions.map((session, idx) => (
                    <tr
                      key={idx}
                      style={{
                        borderBottom: "1px solid #e4e4e430",
                      }}
                    >
                      <td>{session.date}</td>
                      <td>{session.time}</td>
                      <td>
                        {session.volume} Kg{" "}
                        {session.vol_diff != "-" &&
                          "(" + session.vol_diff + ")"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </Modal.Body>
      </Modal>
    </div>
  );
}
