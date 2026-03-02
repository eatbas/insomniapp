import { useState, useEffect, useRef } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getCurrentWindow } from "@tauri-apps/api/window";

const FOUR_HOURS = 4 * 60 * 60 * 1000;
const COOLDOWN = 5 * 60 * 1000;

export function useUpdateCheck() {
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [installing, setInstalling] = useState(false);
  const lastCheckRef = useRef(0);
  const installingRef = useRef(false);
  const attemptedVersionRef = useRef<string | null>(null);

  useEffect(() => {
    const installUpdate = (update: Update) => {
      if (installingRef.current) return;
      if (attemptedVersionRef.current === update.version) return;

      installingRef.current = true;
      attemptedVersionRef.current = update.version;
      setUpdateVersion(update.version);
      setInstalling(true);

      void update
        .downloadAndInstall()
        .then(() => relaunch())
        .catch(() => {
          installingRef.current = false;
          attemptedVersionRef.current = null;
          setInstalling(false);
        });
    };

    const runCheck = () => {
      if (installingRef.current) return;

      const now = Date.now();
      if (now - lastCheckRef.current < COOLDOWN) return;
      lastCheckRef.current = now;

      check()
        .then((u) => {
          if (u?.available) installUpdate(u);
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

  return { installing, updateVersion };
}
