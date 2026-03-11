import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { relaunch } from "@tauri-apps/plugin-process";
import { useTheme } from "../contexts/ThemeContext";
import type { DisguiseState } from "../types";

export default function DisguiseWindow() {
  const { isDark } = useTheme();
  const [apps, setApps] = useState<string[]>([]);
  const [selected, setSelected] = useState<string>("");
  const [state, setState] = useState<DisguiseState | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadApps = useCallback(async () => {
    setError(null);
    setLoading(true);

    try {
      const [disguiseState, runningApps] = await Promise.all([
        invoke<DisguiseState>("get_disguise_state"),
        invoke<string[]>("list_running_apps"),
      ]);

      setState(disguiseState);
      setApps(runningApps);

      if (runningApps.length === 0) {
        setSelected("");
      } else if (!runningApps.includes(selected)) {
        setSelected(runningApps[0]);
      }
    } catch {
      setError("Failed to load running apps. Please try Refresh.");
    } finally {
      setLoading(false);
    }
  }, [selected]);

  useEffect(() => {
    void loadApps();

    // Reload apps each time the window is shown via the disguise button
    const unlisten = listen("refresh-apps", () => {
      void loadApps();
    });
    return () => { void unlisten.then((f) => f()); };
  }, [loadApps]);

  const canApply = useMemo(() => {
    return Boolean(selected) && !busy && !loading && state?.supported;
  }, [busy, loading, selected, state?.supported]);

  const apply = async () => {
    if (!canApply) return;

    setBusy(true);
    setError(null);

    try {
      await invoke("apply_disguise", { name: selected });
      await relaunch();
    } catch {
      setBusy(false);
      setError("Failed to apply disguise name.");
    }
  };

  const reset = async () => {
    if (busy) return;

    setBusy(true);
    setError(null);

    try {
      await invoke("reset_disguise");
      await relaunch();
    } catch {
      setBusy(false);
      setError("Failed to reset disguise name.");
    }
  };

  const containerClass = isDark
    ? "h-screen bg-gray-900 text-white p-3"
    : "h-screen bg-gray-50 text-gray-900 p-3";

  const cardClass = isDark
    ? "rounded border border-gray-800 bg-gray-900"
    : "rounded border border-gray-200 bg-white";

  const mutedClass = isDark ? "text-gray-400" : "text-gray-500";

  if (state && !state.supported) {
    return (
      <div className={containerClass}>
        <div className={cardClass + " p-3 text-xs"}>
          <div className="font-semibold mb-1">Disguise mode is not available on this OS.</div>
          <p className={mutedClass}>This feature currently ships on Windows only.</p>
        </div>
      </div>
    );
  }

  return (
    <div className={containerClass}>
      <div className="flex items-center justify-between mb-2">
        <h1 className="text-sm font-semibold">Disguise Mode</h1>
        <button
          onClick={() => void loadApps()}
          disabled={loading || busy}
          className={`px-2 py-1 text-[11px] rounded transition-colors ${
            isDark
              ? "bg-gray-800 hover:bg-gray-700 text-white disabled:opacity-50"
              : "bg-gray-200 hover:bg-gray-300 text-gray-900 disabled:opacity-50"
          }`}
        >
          Refresh
        </button>
      </div>

      <p className={`text-[11px] mb-2 ${mutedClass}`}>
        Don&apos;t see your app? Open the application, then refresh.
      </p>

      {state && (
        <p className={`text-[11px] mb-2 ${mutedClass}`}>
          Current name: <span className="font-semibold">{state.currentName}</span>
        </p>
      )}

      <div className={cardClass + " h-60 overflow-y-auto p-1"}>
        {loading ? (
          <div className={`text-[11px] p-2 ${mutedClass}`}>Loading running apps...</div>
        ) : apps.length === 0 ? (
          <div className={`text-[11px] p-2 ${mutedClass}`}>
            No visible apps found. Open an app and click Refresh.
          </div>
        ) : (
          <ul className="flex flex-col gap-1">
            {apps.map((appName) => {
              const active = selected === appName;

              return (
                <li key={appName}>
                  <button
                    onClick={() => setSelected(appName)}
                    className={`w-full text-left text-[12px] px-2 py-1 rounded transition-colors ${
                      active
                        ? isDark
                          ? "bg-indigo-600 text-white"
                          : "bg-indigo-500 text-white"
                        : isDark
                          ? "hover:bg-gray-800"
                          : "hover:bg-gray-100"
                    }`}
                  >
                    {appName}
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>

      {error && <p className="text-[11px] text-red-500 mt-2">{error}</p>}

      <div className="mt-3 flex items-center gap-2">
        <button
          onClick={apply}
          disabled={!canApply}
          className="flex-1 px-2 py-1 text-[12px] rounded bg-indigo-600 hover:bg-indigo-700 text-white disabled:opacity-50"
        >
          Apply & Restart
        </button>
        <button
          onClick={reset}
          disabled={busy}
          className={`px-2 py-1 text-[12px] rounded transition-colors ${
            isDark
              ? "bg-gray-800 hover:bg-gray-700 text-white disabled:opacity-50"
              : "bg-gray-200 hover:bg-gray-300 text-gray-900 disabled:opacity-50"
          }`}
        >
          Reset to insomniAPP
        </button>
      </div>
    </div>
  );
}
