import { AppContext } from "@/context/AppContext";
import { AlertType } from "@/models/alert";
import {
  RustBridge,
  SessionDetails,
  SessionSeriesUpdate,
} from "@/utils/RustBridge";
import { useContext, useState } from "react";
import { Button, Modal } from "react-bootstrap";

type Props = {
  session: SessionDetails;
  onClose: () => void;
};

export function SessionModal({ session, onClose }: Props) {
  const { addAlert } = useContext(AppContext);

  const [localSession, setLocalSession] = useState({ ...session });
  const [changed, setChanged] = useState(false);

  const updateSerieReps = (exercise: string, idx: number, newVal: string) => {
    let reps = parseInt(newVal);
    if (isNaN(reps)) {
      reps = 0;
    }
    const newObj = {
      ...localSession,
      series: {
        ...localSession.series,
        [exercise]: localSession.series[exercise].map((serie, id) =>
          id === idx ? { ...serie, reps } : serie,
        ),
      },
    };
    setLocalSession(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(session));
  };

  const updateSerieWeight = (exercise: string, idx: number, newVal: string) => {
    let weight = parseFloat(newVal);
    if (isNaN(weight)) {
      weight = 0;
    }
    const newObj = {
      ...localSession,
      series: {
        ...localSession.series,
        [exercise]: localSession.series[exercise].map((serie, id) =>
          id === idx ? { ...serie, weight } : serie,
        ),
      },
    };
    setLocalSession(newObj);
    setChanged(JSON.stringify(newObj) != JSON.stringify(session));
  };

  const getVolume = () => {
    let volume = 0;
    Object.entries(localSession.series).map(([, series]) => {
      series.forEach((serie) => {
        volume += serie.reps * serie.weight!;
      });
    });

    return volume;
  };

  const saveChanges = () => {
    const update: SessionSeriesUpdate = {
      timestamp: localSession.timestamp,
      series: [],
    };
    Object.entries(localSession.series).forEach(([, series]) => {
      series.forEach((serie) => {
        update.series.push(serie);
      });
    });
    RustBridge.saveSessionChanges(update)
      .then(() => {
        onClose();
      })
      .then(() => {
        addAlert({
          title: "Updated succesful",
          body: "Workout session updated succesfully",
          type: AlertType.INFO,
        });
      })
      .catch((e) => {
        addAlert({
          title: "Error on session update",
          body: e,
          type: AlertType.ERROR,
        });
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
            {localSession.name}
            <small style={{ fontSize: "17px", marginLeft: "30px" }}>
              {localSession.date}
            </small>
          </Modal.Title>
        </Modal.Header>

        <Modal.Body>
          <table id="session-details-table">
            <colgroup>
              <col style={{ width: "200px" }} />
              <col style={{ width: "150px" }} />
              <col />
            </colgroup>
            <tbody>
              <tr>
                <td>Total time:</td>
                <td>{localSession.total_elapsed_time}</td>
              </tr>
              <tr>
                <td>Active time:</td>
                <td>{localSession.active_time}</td>
              </tr>
              <tr>
                <td>Total calories:</td>
                <td>{localSession.total_calories} Kcal</td>
              </tr>
              <tr>
                <td>Active calories:</td>
                <td>
                  {localSession.total_calories -
                    localSession.metabolic_calories}{" "}
                  Kcal
                </td>
              </tr>
              <tr>
                <td>Average heart rate:</td>
                <td> {localSession.avg_heart_rate} BPM</td>
              </tr>
              <tr>
                <td>Max heart rate:</td>
                <td>{localSession.max_heart_rate} BPM</td>
              </tr>
              <tr>
                <td>Volume:</td>
                <td>{getVolume()} Kg</td>
              </tr>
            </tbody>
          </table>
          {Object.keys(localSession.series).length > 0 && (
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
                  {localSession.exercises.map((exercise) =>
                    localSession.series[exercise].map((serie, idx) => (
                      <tr key={`${exercise}-${idx}`}>
                        {idx === 0 && (
                          <td
                            style={{
                              borderBottom:
                                idx === 0 ? "1px solid #e4e4e430" : "",
                            }}
                            rowSpan={localSession.series[exercise].length}
                          >
                            {exercise}
                          </td>
                        )}

                        <td
                          style={{
                            borderBottom:
                              idx === localSession.series[exercise].length - 1
                                ? "1px solid #e4e4e430"
                                : "",
                            paddingBottom:
                              idx === localSession.series[exercise].length - 1
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
                            value={serie.weight?.toString()}
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
