import React from "react";
import ReactDOM from "react-dom/client";
import { MonitorApp } from "./MonitorApp";
import "../widgets/widgets.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MonitorApp />
  </React.StrictMode>,
);
