import { AlertDefinition } from "@/models/alert";
import { Alert } from "react-bootstrap";

type Props = {
  alerts: AlertDefinition[];
};

export function Alerts({ alerts }: Props) {
  return (
    <div id="alert-zone">
      {alerts.map((def, idx) => (
        <Alert key={idx} show={true} variant={def.type}>
          <b>{def.title}</b>
          <div className="alert-body">
            <span>{def.body}</span>
          </div>
        </Alert>
      ))}
    </div>
  );
}
