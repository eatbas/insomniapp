import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../contexts/ThemeContext";
import { Logo, SunIcon, MoonIcon } from "./Icons";
import type { AppStatus } from "../types";

interface Props {
  status: AppStatus;
  appName: string;
  disguiseSupported: boolean;
}

export default function StatusPanel({ status, appName, disguiseSupported }: Props) {
  const { isDark, toggleTheme } = useTheme();

  const handleToggle = async () => {
    await invoke("toggle_enabled");
  };

  const handleOpenDisguise = async () => {
    await invoke("open_disguise_window");
  };

  let label: string;
  let dotColor: string;
  let textColor: string;

  if (!status.enabled) {
    label = "Disabled";
    dotColor = "bg-gray-500";
    textColor = "text-gray-400";
  } else if (status.isSessionLocked) {
    label = "Paused (Locked)";
    dotColor = "bg-yellow-400";
    textColor = "text-yellow-400";
  } else if (status.isDisplayOff) {
    label = "Paused (Screen Off)";
    dotColor = "bg-yellow-400";
    textColor = "text-yellow-400";
  } else if (status.isInMeeting) {
    label = "Paused";
    dotColor = "bg-yellow-400";
    textColor = "text-yellow-400";
  } else if (status.isSimulating) {
    label = "Active";
    dotColor = "bg-green-400 animate-pulse";
    textColor = "text-green-400";
  } else {
    label = "Monitoring";
    dotColor = "bg-blue-400";
    textColor = "text-blue-400";
  }

  return (
    <div className="flex items-center gap-1">
      <Logo />
      <span className={`text-[10px] font-bold shrink-0 ${isDark ? "text-white" : "text-gray-900"}`}>
        {appName}
      </span>
      <div className="flex items-center gap-1">
        <div className={`w-1.5 h-1.5 rounded-full ${dotColor}`} />
        <span className={`text-[10px] font-medium ${textColor}`}>{label}</span>
      </div>
      <div className="flex-1" />
      <button
        onClick={toggleTheme}
        className={`p-0.5 rounded transition-colors ${
          isDark
            ? "text-yellow-400 hover:bg-gray-800"
            : "text-gray-600 hover:bg-gray-200"
        }`}
        title={isDark ? "Switch to light mode" : "Switch to dark mode"}
      >
        {isDark ? <SunIcon /> : <MoonIcon />}
      </button>
      <button
        onClick={handleToggle}
        className={`px-1.5 py-0.5 rounded text-[10px] font-medium transition-colors ${
          status.enabled
            ? "bg-red-600 hover:bg-red-700 text-white"
            : "bg-green-600 hover:bg-green-700 text-white"
        }`}
      >
        {status.enabled ? "Disable" : "Enable"}
      </button>
      {disguiseSupported && (
        <button
          onClick={handleOpenDisguise}
          className={`w-3 h-3 rounded-full transition-colors ${
            isDark
              ? "bg-indigo-500 hover:bg-indigo-400"
              : "bg-indigo-600 hover:bg-indigo-500"
          }`}
          title="Open disguise options"
        />
      )}
    </div>
  );
}
