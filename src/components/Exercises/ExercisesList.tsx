import { ExerciseListItem } from "@/models/exercises";

type Props = {
  exercises: ExerciseListItem[];
  onRowClick: (category: string, id: number) => void;
};

export function ExercisesList({ exercises, onRowClick }: Props) {
  return (
    <table>
      <thead>
        <tr>
          <th style={{ textAlign: "center" }}>Exercise</th>
          <th style={{ textAlign: "center" }}>PR</th>
          <th style={{ textAlign: "center" }}>1RM</th>
        </tr>
      </thead>
      <tbody>
        {exercises.map((exercise, idx) => (
          <tr
            key={idx}
            style={{ cursor: "pointer" }}
            onClick={() => onRowClick(exercise.category, exercise.id)}
          >
            <td style={{ textAlign: "left" }}>{exercise.name}</td>
            <td>{exercise.reps + "x" + exercise.weight + " Kg"}</td>
            <td>{Math.floor(exercise.rm) + " Kg"}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
