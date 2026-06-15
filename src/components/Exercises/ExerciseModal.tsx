import { ExerciseDetails } from "@/models/exercises";
import { Modal } from "react-bootstrap";

type Props = {
  exercise: ExerciseDetails;
  onClose: () => void;
};

export function ExerciseModal({ exercise, onClose }: Props) {
  return (
    <div
      className="modal show"
      style={{ display: "block", position: "initial" }}
    >
      <Modal show={true} onHide={onClose} data-bs-theme="dark">
        <Modal.Header closeButton>
          <Modal.Title>{exercise.name}</Modal.Title>
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
                <td>Personal record:</td>
                <td>{exercise.reps + "x" + exercise.weight + " Kg"}</td>
              </tr>
              <tr>
                <td>1 RM:</td>
                <td>{Math.floor(exercise.rm) + " Kg"}</td>
              </tr>
            </tbody>
          </table>
          {Object.keys(exercise.workouts).length > 0 && (
            <>
              <hr />
              <table>
                <colgroup>
                  <col style={{ width: "350px" }} />
                  <col style={{ width: "150px" }} />
                </colgroup>

                <thead>
                  <tr>
                    <th>Workout</th>
                    <th>Series</th>
                  </tr>
                </thead>
                <tbody>
                  {Object.entries(exercise.workouts).map(([workout, series]) =>
                    series.map((serie, idx) => (
                      <tr key={`${workout}-${idx}`}>
                        {idx === 0 && (
                          <td
                            style={{
                              borderBottom:
                                idx === 0 ? "1px solid #e4e4e430" : "",
                            }}
                            rowSpan={series.length}
                          >
                            {workout.split("\n")[0]}
                            <br />
                            {workout.split("\n")[1]}
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
