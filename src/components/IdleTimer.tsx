import { useTheme } from "../contexts/ThemeContext";
import type { AppStatus } from "../types";

interface Props {
  status: AppStatus;
}

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export default function IdleTimer({ status }: Props) {
  const { isDark } = useTheme();

  const progress = Math.min(
    (status.idleSeconds / status.idleThresholdSecs) * 100,
    100,
  );

  return (
    <div className="flex items-center gap-1 flex-1 min-w-0">
      <span className={`text-[10px] shrink-0 ${isDark ? "text-gray-500" : "text-gray-500"}`}>Idle</span>
      <span className={`text-[10px] font-mono shrink-0 ${isDark ? "text-white" : "text-gray-900"}`}>
        {formatTime(status.idleSeconds)}
      </span>
      <div className={`flex-1 rounded-full h-1.5 min-w-6 ${isDark ? "bg-gray-700" : "bg-gray-300"}`}>
        <div
          className={`h-1.5 rounded-full transition-all duration-1000 ${
            status.isIdle ? "bg-orange-500" : "bg-blue-500"
          }`}
          style={{ width: `${progress}%` }}
        />
      </div>
      <span className={`text-[10px] shrink-0 ${isDark ? "text-gray-500" : "text-gray-500"}`}>
        {formatTime(status.idleThresholdSecs)}
      </span>
    </div>
  );
}
