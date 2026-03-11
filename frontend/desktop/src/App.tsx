import { ThemeProvider, useTheme } from "./contexts/ThemeContext";
import { useAppState } from "./hooks/useAppState";
import { useDisguiseState } from "./hooks/useDisguiseState";
import { useUpdateCheck } from "./hooks/useUpdateCheck";
import StatusPanel from "./components/StatusPanel";
import IdleTimer from "./components/IdleTimer";
import MeetingIndicator from "./components/MeetingIndicator";
import SettingsForm from "./components/SettingsForm";

function AppContent() {
  const status = useAppState();
  const disguise = useDisguiseState();
  const { isDark } = useTheme();
  const { installing, updateVersion } = useUpdateCheck();

  if (!status) {
    return (
      <div className={`h-screen flex items-center justify-center ${isDark ? "bg-gray-900 text-white" : "bg-gray-50 text-gray-900"}`}>
        <div className="text-gray-500 text-xs">Loading...</div>
      </div>
    );
  }

  return (
    <div className={`min-h-screen px-2 pt-1 pb-0.5 flex flex-col gap-0.5 ${isDark ? "bg-gray-900 text-white" : "bg-gray-50 text-gray-900"}`}>
      <StatusPanel
        status={status}
        appName={disguise.currentName}
        disguiseSupported={disguise.supported}
      />
      <div className="flex items-center gap-1.5">
        <IdleTimer status={status} />
        <MeetingIndicator status={status} />
      </div>
      <SettingsForm status={status} />
      {installing && (
        <div className="w-full text-[9px] py-0.5 rounded text-center font-medium bg-gray-500 text-white">
          {`Updating to v${updateVersion ?? "latest"}...`}
        </div>
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
