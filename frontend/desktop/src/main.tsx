import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "./App";
import { ThemeProvider } from "./contexts/ThemeContext";
import DisguiseWindow from "./components/DisguiseWindow";
import "./styles.css";

let currentLabel = "unknown";
try {
  currentLabel = getCurrentWebviewWindow().label;
} catch (err) {
  void invoke("debug_log", {
    message: `main.tsx failed to read current window label: ${String(err)}`,
  });
}

void invoke("debug_log", {
  message: `main.tsx boot label=${currentLabel} href=${window.location.href}`,
});

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
