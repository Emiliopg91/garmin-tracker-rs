import { WorkoutDetails, WorkoutSeriesUpdate } from "@/models/workouts";
import { RpcUtils } from "@/utils/RpcUtils";
import { useState } from "react";
import { Button, Modal } from "react-bootstrap";

type Props = {
  workout: WorkoutDetails;
  onClose: () => void;
};

export function WorkoutModal({ workout, onClose }: Props) {
  const [localWorkout, setLocalWorkout] = useState({ ...workout });
  const [changed, setChanged] = useState(false);

  const updateSerieReps = (exercise: string, idx: number, newVal: string) => {
    let reps = parseInt(newVal);
    if (isNaN(reps)) {
      reps = 0;
    }
    const newObj = {
      ...localWorkout,
      series: {
        ...localWorkout.series,
        [exercise]: localWorkout.series[exercise].map((serie, id) =>
          id === idx ? { ...serie, reps } : serie,
        ),
      },
    };
    setLocalWorkout(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(workout));
  };

  const updateSerieWeight = (exercise: string, idx: number, newVal: string) => {
    let weight = parseFloat(newVal);
    if (isNaN(weight)) {
      weight = 0;
    }
    const newObj = {
      ...localWorkout,
      series: {
        ...localWorkout.series,
        [exercise]: localWorkout.series[exercise].map((serie, id) =>
          id === idx ? { ...serie, weight } : serie,
        ),
      },
    };
    setLocalWorkout(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(workout));
  };

  const getVolume = () => {
    let volume = 0;
    Object.entries(localWorkout.series).map(([_, series]) => {
      series.forEach((serie) => {
        volume += serie.reps * serie.weight;
      });
    });

    return volume;
  };

  const saveChanges = () => {
    const update: WorkoutSeriesUpdate = {
      timestamp: localWorkout.timestamp,
      series: [],
    };
    Object.entries(localWorkout.series).forEach(([_, series]) => {
      series.forEach((serie) => {
        update.series.push(serie);
      });
    });
    RpcUtils.saveWorkoutChanges(update).then(() => {
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
          <Modal.Title>
            {localWorkout.name}
            <small style={{ fontSize: "17px", marginLeft: "30px" }}>
              {localWorkout.date}
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
                <td>{localWorkout.total_elapsed_time}</td>
              </tr>
              <tr>
                <td>Active time:</td>
                <td>{localWorkout.active_time}</td>
              </tr>
              <tr>
                <td>Total calories:</td>
                <td>{localWorkout.total_calories} Kcal</td>
              </tr>
              <tr>
                <td>Active calories:</td>
                <td>
                  {localWorkout.total_calories - workout.metabolic_calories}{" "}
                  Kcal
                </td>
              </tr>
              <tr>
                <td>Average heart rate:</td>
                <td> {localWorkout.avg_heart_rate} BPM</td>
              </tr>
              <tr>
                <td>Max heart rate:</td>
                <td>{localWorkout.max_heart_rate} BPM</td>
              </tr>
              <tr>
                <td>Volume:</td>
                <td>{getVolume()} Kg</td>
              </tr>
            </tbody>
          </table>
          {Object.keys(localWorkout.series).length > 0 && (
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
                  {localWorkout.exercises.map((exercise) =>
                    localWorkout.series[exercise].map((serie, idx) => (
                      <tr key={`${exercise}-${idx}`}>
                        {idx === 0 && (
                          <td
                            style={{
                              borderBottom:
                                idx === 0 ? "1px solid #e4e4e430" : "",
                            }}
                            rowSpan={localWorkout.series[exercise].length}
                          >
                            {exercise}
                          </td>
                        )}

                        <td
                          style={{
                            borderBottom:
                              idx === localWorkout.series[exercise].length - 1
                                ? "1px solid #e4e4e430"
                                : "",
                            paddingBottom:
                              idx === localWorkout.series[exercise].length - 1
                                ? "5px"
                                : "",
                            paddingTop: idx === 0 ? "5px" : "",
                          }}
                        >
                          <input
                            type="number"
                            value={serie.reps}
                            className="no-spinner"
                            min={0}
                            style={{ width: "2em", textAlign: "center" }}
                            onChange={(e) => {
                              updateSerieReps(exercise, idx, e.target.value);
                            }}
                          />{" "}
                          x{" "}
                          <input
                            type="number"
                            value={serie.weight}
                            className="no-spinner"
                            min={0}
                            style={{ width: "3em", textAlign: "center" }}
                            onChange={(e) => {
                              updateSerieWeight(exercise, idx, e.target.value);
                            }}
                          />
                          Kg
                        </td>
                      </tr>
                    )),
                  )}
                </tbody>
              </table>
              <div style={{ padding: "5px" }}>
                <Button
                  id="import-button"
                  disabled={!changed}
                  style={{ width: "100%" }}
                  onClick={saveChanges}
                >
                  Update sets
                </Button>
              </div>
            </>
          )}
        </Modal.Body>
      </Modal>
    </div>
  );
}
