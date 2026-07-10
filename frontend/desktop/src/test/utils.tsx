import { render } from "@testing-library/react";
import type { ReactElement } from "react";
import { ThemeProvider } from "../contexts/ThemeContext";
import type { AppStatus } from "../types";

type Theme = "dark" | "light";

// Builds a fully-populated AppStatus, letting each test override only the
// fields relevant to the behaviour under test.
export function makeStatus(overrides: Partial<AppStatus> = {}): AppStatus {
  return {
    enabled: true,
    isIdle: false,
    idleSeconds: 0,
    isSessionLocked: false,
    isDisplayOff: false,
    isSimulating: false,
    idleThresholdSecs: 30,
    simulationIntervalSecs: 15,
    ...overrides,
  };
}

// Renders a subtree inside the real ThemeProvider. The provider reads the
// initial theme from localStorage exactly once, so seeding storage before the
// render lets a test choose dark or light mode deterministically.
export function renderWithTheme(ui: ReactElement, theme: Theme = "dark") {
  localStorage.setItem("insomniapp-theme", theme);
  return render(<ThemeProvider>{ui}</ThemeProvider>);
}
