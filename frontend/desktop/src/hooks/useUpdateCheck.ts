import { useState, useEffect } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export function useUpdateCheck() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    check().then((u) => {
      if (u?.available) setUpdate(u);
    }).catch(() => {});
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
