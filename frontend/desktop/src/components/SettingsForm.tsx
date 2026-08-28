import { useRef, useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { useTheme } from "../contexts/ThemeContext";
import type { AppStatus, NudgeMethod, SettingsPayload } from "../types";

interface Props {
  status: AppStatus;
}

/** The two numeric settings, both edited through the same debounced path. */
type IntervalField = "idleThresholdSecs" | "simulationIntervalSecs";

export default function SettingsForm({ status }: Props) {
  const { isDark } = useTheme();
  const debounceRef = useRef<number | null>(null);
  const [version, setVersion] = useState("");

  useEffect(() => {
    // The version label is cosmetic, so a failure to resolve it leaves the
    // label hidden rather than surfacing an error in a 78px window.
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(""));

    return () => {
      if (debounceRef.current !== null) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  // One debounce slot is shared by every control: the form only ever edits a
  // single setting at a time, and the backend re-broadcasts the authoritative
  // status every tick, so a superseded or rejected update needs no local
  // rollback — only a handled rejection.
  const sendSettings = (settings: SettingsPayload) => {
    if (debounceRef.current !== null) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = window.setTimeout(() => {
      debounceRef.current = null;
      invoke("update_settings", { settings }).catch(() => undefined);
    }, 500);
  };

  const handleIntervalChange = (field: IntervalField, value: string) => {
    const num = parseInt(value, 10);
    if (isNaN(num) || num < 1) return;

    sendSettings({ [field]: num });
  };

  const controlClass = isDark
    ? "bg-gray-800 border border-gray-700 text-white focus:border-blue-500"
    : "bg-white border border-gray-300 text-gray-900 focus:border-blue-500";

  const inputClass = `w-10 rounded px-1 py-0.5 text-[10px] focus:outline-none ${controlClass}`;
  const selectClass = `w-14 rounded px-1 py-0.5 text-[10px] focus:outline-none ${controlClass}`;

  return (
    <div className={`flex items-center gap-2 border-t pt-0.5 ${isDark ? "border-gray-800" : "border-gray-200"}`}>
      <div className="flex items-center gap-1">
        <label className="text-[10px] text-gray-500">Idle:</label>
        <input
          type="number"
          min="10"
          max="600"
          defaultValue={status.idleThresholdSecs}
          onChange={(e) => handleIntervalChange("idleThresholdSecs", e.target.value)}
          className={inputClass}
        />
        <span className={`text-[10px] ${isDark ? "text-gray-600" : "text-gray-400"}`}>s</span>
      </div>
      <div className="flex items-center gap-1">
        <label className="text-[10px] text-gray-500">Interval:</label>
        <input
          type="number"
          min="5"
          max="300"
          defaultValue={status.simulationIntervalSecs}
          onChange={(e) =>
            handleIntervalChange("simulationIntervalSecs", e.target.value)
          }
          className={inputClass}
        />
        <span className={`text-[10px] ${isDark ? "text-gray-600" : "text-gray-400"}`}>s</span>
      </div>
      {/* The options name themselves, so the control carries no visible label:
          the row has no width to spare. Uncontrolled, like the two inputs above,
          because the backend re-broadcasts the status every tick and a
          controlled value would snap back to the old method for the length of
          the debounce. */}
      <div className="flex items-center gap-1">
        <select
          aria-label="Nudge method"
          title="How the idle counter is reset: a silent pointer move, or an F15 keypress"
          defaultValue={status.nudgeMethod}
          onChange={(e) =>
            sendSettings({ nudgeMethod: e.target.value as NudgeMethod })
          }
          className={selectClass}
        >
          <option value="mouseNudge">Mouse</option>
          <option value="f15">F15</option>
        </select>
      </div>
      <div className="flex-1" />
      {version && (
        <span className={`text-[8px] ${isDark ? "text-gray-600" : "text-gray-400"}`}>
          v{version}
        </span>
      )}
    </div>
  );
}
