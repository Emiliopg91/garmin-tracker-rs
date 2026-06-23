import ReactDOM from "react-dom/client";
import App from "@/components/App/App";

import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap/dist/js/bootstrap.bundle.min";

import "@/styles/main.css";
import { AppProvider } from "./context/AppProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <AppProvider>
    <App />
  </AppProvider>,
);
