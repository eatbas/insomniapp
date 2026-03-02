import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../contexts/ThemeContext";
import type { AppStatus } from "../types";

interface Props {
  status: AppStatus;
}

function Logo() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 1024 1024" fill="none">
      <defs>
        <linearGradient id="outer" x1="128" y1="160" x2="896" y2="960" gradientUnits="userSpaceOnUse">
          <stop stopColor="#2384FF"/>
          <stop offset="1" stopColor="#0E4CC9"/>
        </linearGradient>
        <linearGradient id="screen" x1="512" y1="260" x2="512" y2="690" gradientUnits="userSpaceOnUse">
          <stop stopColor="#E7F5FF"/>
          <stop offset="1" stopColor="#C8E8FF"/>
        </linearGradient>
        <linearGradient id="stand" x1="300" y1="850" x2="724" y2="930" gradientUnits="userSpaceOnUse">
          <stop stopColor="#3C74D9"/>
          <stop offset="1" stopColor="#1E4FAE"/>
        </linearGradient>
      </defs>
      <g>
        <rect x="90" y="165" width="844" height="650" rx="118" fill="url(#outer)" stroke="#80D1FF" strokeWidth="18"/>
        <rect x="145" y="220" width="734" height="538" rx="84" fill="url(#screen)"/>
        <path d="M300 510C335 458 403 458 438 510" stroke="#21314E" strokeWidth="34" strokeLinecap="round"/>
        <path d="M586 510C621 458 689 458 724 510" stroke="#21314E" strokeWidth="34" strokeLinecap="round"/>
        <path d="M454 622C488 654 536 654 570 622" stroke="#2A3D62" strokeWidth="24" strokeLinecap="round"/>
        <rect x="430" y="780" width="164" height="70" rx="26" fill="#245BC4"/>
        <rect x="300" y="850" width="424" height="80" rx="32" fill="url(#stand)"/>
      </g>
      <path d="M720 110L792 110L745 176L811 176" stroke="#0D57E0" strokeWidth="28" strokeLinecap="round" strokeLinejoin="round"/>
      <path d="M645 138L696 138L662 186L709 186" stroke="#2C7BFF" strokeWidth="20" strokeLinecap="round" strokeLinejoin="round"/>
    </svg>
  );
}

function SunIcon() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="5"/>
      <line x1="12" y1="1" x2="12" y2="3"/>
      <line x1="12" y1="21" x2="12" y2="23"/>
      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
      <line x1="1" y1="12" x2="3" y2="12"/>
      <line x1="21" y1="12" x2="23" y2="12"/>
      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
    </svg>
  );
}

export default function StatusPanel({ status }: Props) {
  const { isDark, toggleTheme } = useTheme();

  const handleToggle = async () => {
    await invoke("toggle_enabled");
  };

  let label: string;
  let dotColor: string;
  let textColor: string;

  if (!status.enabled) {
    label = "Disabled";
    dotColor = "bg-gray-500";
    textColor = "text-gray-400";
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
        insomniAPP
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
    </div>
  );
}
