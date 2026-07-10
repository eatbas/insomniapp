import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { AppStatus } from "../types";

export function useAppState() {
  const [status, setStatus] = useState<AppStatus | null>(null);

  useEffect(() => {
    invoke<AppStatus>("get_status")
      .then(setStatus)
      .catch(() => {
        // Keep the loading state if the initial status cannot be read; a
        // subsequent `status-update` event will populate it.
      });

    const unlisten = listen<AppStatus>("status-update", (event) => {
      setStatus(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return status;
}
