import { WorkoutListItem } from "@/utils/RustBridge";

type Props = {
  workouts: WorkoutListItem[];
  onRowClick: (timestamp: number) => void;
};

export function WorkoutsList({ workouts, onRowClick }: Props) {
  return (
    <table>
      <thead>
        <tr>
          <th style={{ textAlign: "center" }}>Exercise</th>
          <th style={{ textAlign: "center" }}>Date</th>
        </tr>
      </thead>

      <tbody>
        {workouts.map((workout, idx) => (
          <tr
            key={idx}
            onClick={() => onRowClick(workout.timestamp)}
            style={{ cursor: "pointer" }}
          >
            <td>{workout.name}</td>
            <td>{workout.date}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
