import { useState, useEffect, useRef } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getCurrentWindow } from "@tauri-apps/api/window";

const FOUR_HOURS = 4 * 60 * 60 * 1000;
const COOLDOWN = 5 * 60 * 1000;

export function useUpdateCheck() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [installing, setInstalling] = useState(false);
  const lastCheckRef = useRef(0);

  useEffect(() => {
    const runCheck = () => {
      const now = Date.now();
      if (now - lastCheckRef.current < COOLDOWN) return;
      lastCheckRef.current = now;

      check()
        .then((u) => {
          if (u?.available) setUpdate(u);
        })
        .catch(() => {});
    };

    // Check on mount
    runCheck();

    // Check every 4 hours
    const interval = setInterval(runCheck, FOUR_HOURS);

    // Check when window gains focus (user opens from tray)
    const unlisten = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) runCheck();
    });

    return () => {
      clearInterval(interval);
      unlisten.then((fn) => fn());
    };
  }, []);

  const install = async () => {
    if (!update) return;
    setInstalling(true);
    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch {
      setInstalling(false);
    }
  };

  return { update, installing, install };
}
