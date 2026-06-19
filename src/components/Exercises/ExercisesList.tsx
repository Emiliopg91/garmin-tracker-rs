import { ExerciseListItem } from "@/utils/backend/models";

type Props = {
  exercises: ExerciseListItem[];
  onRowClick: (category: string, id: number) => void;
};

export function ExercisesList({ exercises, onRowClick }: Props) {
  return (
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
            onClick={() => onRowClick(exercise.category, exercise.id)}
          >
            <td style={{ textAlign: "left" }}>{exercise.name}</td>
            <td>{exercise.reps + "x" + exercise.weight + " Kg"}</td>
            <td>{Math.floor(exercise.rm!) + " Kg"}</td>
            <td>{exercise.date.split(" ")[1]}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
