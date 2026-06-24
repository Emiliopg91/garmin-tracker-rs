import { ExerciseDetails, ExerciseListItem } from "@/utils/backend/models";
import { ExerciseModal } from "./ExerciseModal";
import { useContext, useEffect, useState } from "react";
import { BackendClient } from "@/utils/backend/client";
import { AppContext } from "@/context/AppContext";

export function ExercisesList() {
  const { setLoading } = useContext(AppContext);
  const [exercises, setExercises] = useState<ExerciseListItem[]>([]);
  const [exerciseDetails, setExerciseDetails] = useState<
    ExerciseDetails | undefined
  >(undefined);

  const refreshList = () => {
    BackendClient.getExercises()
      .then((data) => {
        setExercises(data);
      })
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    refreshList();
  }, []);

  const getExerciseDetails = (category: string, id: number) => {
    BackendClient.getExerciseDetails(category, id).then((details) => {
      setExerciseDetails(details);
    });
  };

  return (
    <>
      <table>
        <colgroup>
          <col style={{ width: "400px" }} />
          <col style={{ width: "100px" }} />
          <col style={{ width: "60px" }} />
          <col style={{ width: "100px" }} />
        </colgroup>
        <thead>
          <tr>
            <th style={{ textAlign: "center" }}>Exercise</th>
            <th style={{ textAlign: "center" }}>PR</th>
            <th style={{ textAlign: "center" }}>1RM</th>
            <th style={{ textAlign: "center" }}>Date</th>
          </tr>
        </thead>
        <tbody>
          {exercises.map((exercise, idx) => (
            <tr
              key={idx}
              style={{ cursor: "pointer" }}
              onClick={() => getExerciseDetails(exercise.category, exercise.id)}
            >
              <td style={{ textAlign: "left" }}>{exercise.name}</td>
              <td>{exercise.reps + "x" + exercise.weight + " Kg"}</td>
              <td>{Math.floor(exercise.rm!) + " Kg"}</td>
              <td>{exercise.date.split(" ")[1]}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div>
        {exerciseDetails && (
          <ExerciseModal
            exercise={exerciseDetails}
            onClose={() => setExerciseDetails(undefined)}
          />
        )}
      </div>
    </>
  );
}
