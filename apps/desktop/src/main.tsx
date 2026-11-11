import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
import "@fontsource/inter/600.css";
import "./styles/global.css";
import App from "./App";
import { appendCrashLog } from "./lib/tauriApi";

function reportClientCrash(message: string) {
  void appendCrashLog(message).catch(() => {
    /* local only — ignore if backend unavailable */
  });
}

window.addEventListener("error", (event) => {
  reportClientCrash(
    `frontend error: ${event.message} @ ${event.filename}:${event.lineno}`,
  );
});

window.addEventListener("unhandledrejection", (event) => {
  const reason =
    event.reason instanceof Error
      ? event.reason.message
      : String(event.reason);
  reportClientCrash(`frontend rejection: ${reason}`);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
