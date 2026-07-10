import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";

// Unmount React trees, clear persisted theme state, and reset mocks between
// tests so component state, timers, and the Tauri IPC spies configured per file
// never leak across cases.
afterEach(() => {
  cleanup();
  localStorage.clear();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});
