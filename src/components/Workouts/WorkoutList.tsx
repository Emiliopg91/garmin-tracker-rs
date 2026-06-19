import { WorkoutListItem } from "@/utils/backend/models";

type Props = {
  workouts: WorkoutListItem[];
  onRowClick: (name: string) => void;
};

export function WorkoutsList({ workouts, onRowClick }: Props) {
  return (
    <table>
      <thead>
        <tr>
          <th style={{ textAlign: "center" }}>Workout</th>
          <th style={{ textAlign: "center" }}>Latest session</th>
          <th style={{ textAlign: "center" }}>Session count</th>
          <th style={{ textAlign: "center" }}>Average duration</th>
        </tr>
      </thead>
      <tbody>
        {workouts.map((workout, idx) => (
          <tr
            key={idx}
            style={{ cursor: "pointer" }}
            onClick={() => onRowClick(workout.name)}
          >
            <td style={{ textAlign: "left" }}>{workout.name}</td>
            <td>{workout.latest_session}</td>
            <td>{workout.sessions}</td>
            <td>{workout.avg_time}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
