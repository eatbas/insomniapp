import { ThemeProvider, useTheme } from "./contexts/ThemeContext";
import { useAppState } from "./hooks/useAppState";
import { useUpdateCheck } from "./hooks/useUpdateCheck";
import StatusPanel from "./components/StatusPanel";
import IdleTimer from "./components/IdleTimer";
import MeetingIndicator from "./components/MeetingIndicator";
import SettingsForm from "./components/SettingsForm";

function AppContent() {
  const status = useAppState();
  const { isDark } = useTheme();
  const { update, installing, install } = useUpdateCheck();

  if (!status) {
    return (
      <div className={`h-screen flex items-center justify-center ${isDark ? "bg-gray-900 text-white" : "bg-gray-50 text-gray-900"}`}>
        <div className="text-gray-500 text-xs">Loading...</div>
      </div>
    );
  }

  return (
    <div className={`min-h-screen px-2 pt-1 pb-0.5 flex flex-col gap-0.5 ${isDark ? "bg-gray-900 text-white" : "bg-gray-50 text-gray-900"}`}>
      <StatusPanel status={status} />
      <div className="flex items-center gap-1.5">
        <IdleTimer status={status} />
        <MeetingIndicator status={status} />
      </div>
      <SettingsForm status={status} />
      {update && (
        <button
          onClick={install}
          disabled={installing}
          className={`w-full text-[9px] py-0.5 rounded text-center font-medium cursor-pointer ${
            installing
              ? "bg-gray-500 text-white"
              : "bg-emerald-600 hover:bg-emerald-700 text-white"
          }`}
        >
          {installing ? "Updating..." : `Update v${update.version} available`}
        </button>
      )}
    </div>
  );
}

function App() {
  return (
    <ThemeProvider>
      <AppContent />
    </ThemeProvider>
  );
}

export default App;
