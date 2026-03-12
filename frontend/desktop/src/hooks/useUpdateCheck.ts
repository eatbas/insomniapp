import { useState, useEffect, useRef } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getCurrentWindow } from "@tauri-apps/api/window";

const CHECK_INTERVAL = 4 * 60 * 60 * 1000;
const CHECK_COOLDOWN = 5 * 60 * 1000;

interface UpdateState {
  lastCheck: number;
  installing: boolean;
  attemptedVersion: string | null;
}

export function useUpdateCheck() {
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [installing, setInstalling] = useState(false);
  const ref = useRef<UpdateState>({ lastCheck: 0, installing: false, attemptedVersion: null });

  useEffect(() => {
    const installUpdate = (update: Update) => {
      const s = ref.current;
      if (s.installing || s.attemptedVersion === update.version) return;

      s.installing = true;
      s.attemptedVersion = update.version;
      setUpdateVersion(update.version);
      setInstalling(true);

      void update
        .downloadAndInstall()
        .then(() => relaunch())
        .catch(() => {
          s.installing = false;
          s.attemptedVersion = null;
          setInstalling(false);
        });
    };

    const runCheck = () => {
      const s = ref.current;
      if (s.installing) return;

      const now = Date.now();
      if (now - s.lastCheck < CHECK_COOLDOWN) return;
      s.lastCheck = now;

      check()
        .then((u) => {
          if (u?.available) installUpdate(u);
        })
        .catch(() => {});
    };

    runCheck();

    const interval = setInterval(runCheck, CHECK_INTERVAL);

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
