import ReactDOM from "react-dom/client";
import App from "@/components/App/App";

import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap/dist/js/bootstrap.bundle.min";

import "@/styles/main.css";
import { AppProvider } from "./context/AppProvider";
import { LogLevel } from "./utils/backend/models";
import { BackendClient } from "./utils/backend/client";

const levels = ["log", "debug", "info", "warn", "error"] as const;

const backendLevel: Record<(typeof levels)[number], LogLevel> = {
  log: LogLevel.Info,
  debug: LogLevel.Debug,
  info: LogLevel.Info,
  warn: LogLevel.Warn,
  error: LogLevel.Error,
};

for (const level of levels) {
  const original = console[level];

  console[level] = (...args: unknown[]) => {
    original(...args);
    BackendClient.logFromFrontend(
      backendLevel[level],
      args
        .map((arg) =>
          typeof arg === "string" ? arg : JSON.stringify(arg, null, 2),
        )
        .join(" "),
    );
  };
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <AppProvider>
    <App />
  </AppProvider>,
);
