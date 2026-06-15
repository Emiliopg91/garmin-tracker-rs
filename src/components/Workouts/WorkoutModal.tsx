import { WorkoutDetails } from "@/models/workouts";
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
          <Modal.Title>
            {workout.name}
            <small style={{ fontSize: "17px", marginLeft: "30px" }}>
              {workout.date}
            </small>
          </Modal.Title>
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
                <td>Total time:</td>
                <td>{workout.total_elapsed_time}</td>
              </tr>
              <tr>
                <td>Active time:</td>
                <td>{workout.active_time}</td>
              </tr>
              <tr>
                <td>Total calories:</td>
                <td>{workout.total_calories} Kcal</td>
              </tr>
              <tr>
                <td>Active calories:</td>
                <td>
                  {workout.total_calories - workout.metabolic_calories} Kcal
                </td>
              </tr>
              <tr>
                <td>Average heart rate:</td>
                <td> {workout.avg_heart_rate} BPM</td>
              </tr>
              <tr>
                <td>Max heart rate:</td>
                <td>{workout.max_heart_rate} BPM</td>
              </tr>
              <tr>
                <td>Volume:</td>
                <td>{workout.volume} Kg</td>
              </tr>
            </tbody>
          </table>
          {Object.keys(workout.series).length > 0 && (
            <>
              <hr />
              <table>
                <colgroup>
                  <col style={{ width: "350px" }} />
                  <col style={{ width: "150px" }} />
                </colgroup>

                <thead>
                  <tr>
                    <th>Exercise</th>
                    <th>Series</th>
                  </tr>
                </thead>
                <tbody>
                  {Object.entries(workout.series).map(([exercise, series]) =>
                    series.map((serie, idx) => (
                      <tr key={`${exercise}-${idx}`}>
                        {idx === 0 && (
                          <td
                            style={{
                              borderBottom:
                                idx === 0 ? "1px solid #e4e4e430" : "",
                            }}
                            rowSpan={series.length}
                          >
                            {exercise}
                          </td>
                        )}

                        <td
                          style={{
                            borderBottom:
                              idx === series.length - 1
                                ? "1px solid #e4e4e430"
                                : "",
                          }}
                        >
                          {serie.reps}x{serie.weight} Kg
                        </td>
                      </tr>
                    )),
                  )}
                </tbody>
              </table>
            </>
          )}
        </Modal.Body>
      </Modal>
    </div>
  );
}
