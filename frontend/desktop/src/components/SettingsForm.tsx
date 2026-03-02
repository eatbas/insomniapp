import { useRef, useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { useTheme } from "../contexts/ThemeContext";
import type { AppStatus } from "../types";

interface Props {
  status: AppStatus;
}

export default function SettingsForm({ status }: Props) {
  const { isDark } = useTheme();
  const debounceRef = useRef<number | null>(null);
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  const handleChange = (field: string, value: string) => {
    const num = parseInt(value, 10);
    if (isNaN(num) || num < 1) return;

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = window.setTimeout(() => {
      invoke("update_settings", {
        settings: { [field]: num },
      });
    }, 500);
  };

  const inputClass = `w-10 rounded px-1 py-0.5 text-[10px] focus:outline-none ${
    isDark
      ? "bg-gray-800 border border-gray-700 text-white focus:border-blue-500"
      : "bg-white border border-gray-300 text-gray-900 focus:border-blue-500"
  }`;

  return (
    <div className={`flex items-center gap-2 border-t pt-0.5 ${isDark ? "border-gray-800" : "border-gray-200"}`}>
      <div className="flex items-center gap-1">
        <label className="text-[10px] text-gray-500">Idle:</label>
        <input
          type="number"
          min="10"
          max="600"
          defaultValue={status.idleThresholdSecs}
          onChange={(e) => handleChange("idleThresholdSecs", e.target.value)}
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
            handleChange("simulationIntervalSecs", e.target.value)
          }
          className={inputClass}
        />
        <span className={`text-[10px] ${isDark ? "text-gray-600" : "text-gray-400"}`}>s</span>
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
