import { SessionListItem } from "@/utils/backend/models";

type Props = {
  sessions: SessionListItem[];
  onRowClick: (timestamp: string) => void;
};

export function SessionsList({ sessions, onRowClick }: Props) {
  return (
    <table>
      <thead>
        <tr>
          <th style={{ textAlign: "center" }}>Workout</th>
          <th style={{ textAlign: "center" }}>Date</th>
          <th style={{ textAlign: "center" }}>Exercises</th>
          <th style={{ textAlign: "center" }}>Sets</th>
          <th style={{ textAlign: "center" }}>Volume</th>
        </tr>
      </thead>

      <tbody>
        {sessions.map((session, idx) => (
          <tr
            key={idx}
            onClick={() => onRowClick(session.timestamp)}
            style={{ cursor: "pointer" }}
          >
            <td>{session.name}</td>
            <td>{session.date}</td>
            <td>{session.exercises_num}</td>
            <td>{session.series_num}</td>
            <td>{session.volume} Kg</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
