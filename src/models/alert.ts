export enum AlertType {
  INFO = "success",
  WARN = "warning",
  ERROR = "danger",
}

export const ALERT_DURATION: Record<AlertType, number> = {
  [AlertType.INFO]: 3000,
  [AlertType.WARN]: 3000,
  [AlertType.ERROR]: 5000,
};

export interface AlertDefinition {
  title: string | undefined;
  body: string;
  type: AlertType;
}
