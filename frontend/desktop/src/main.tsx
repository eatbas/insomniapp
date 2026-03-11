import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "./App";
import { ThemeProvider } from "./contexts/ThemeContext";
import DisguiseWindow from "./components/DisguiseWindow";
import "./styles.css";

let currentLabel = "unknown";
try {
  currentLabel = getCurrentWebviewWindow().label;
} catch {
  // ignore — falls back to "unknown" which renders App
}

const isDisguiseWindow = currentLabel === "disguise";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isDisguiseWindow ? (
      <ThemeProvider>
        <DisguiseWindow />
      </ThemeProvider>
    ) : (
      <App />
    )}
  </React.StrictMode>,
);
