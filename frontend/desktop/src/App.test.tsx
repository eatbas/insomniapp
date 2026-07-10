import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import App from "./App";
import { makeStatus } from "./test/utils";
import type { AppStatus, DisguiseState } from "./types";

vi.mock("./hooks/useAppState", () => ({ useAppState: vi.fn() }));
vi.mock("./hooks/useDisguiseState", () => ({ useDisguiseState: vi.fn() }));
vi.mock("./hooks/useUpdateCheck", () => ({ useUpdateCheck: vi.fn() }));
vi.mock("./components/StatusPanel", () => ({
  default: () => <div data-testid="status-panel" />,
}));
vi.mock("./components/IdleTimer", () => ({
  default: () => <div data-testid="idle-timer" />,
}));
vi.mock("./components/SettingsForm", () => ({
  default: () => <div data-testid="settings-form" />,
}));

import { useAppState } from "./hooks/useAppState";
import { useDisguiseState } from "./hooks/useDisguiseState";
import { useUpdateCheck } from "./hooks/useUpdateCheck";

const useAppStateMock = vi.mocked(useAppState);
const useDisguiseStateMock = vi.mocked(useDisguiseState);
const useUpdateCheckMock = vi.mocked(useUpdateCheck);

const disguise: DisguiseState = {
  supported: true,
  currentName: "insomniAPP",
  isDisguised: false,
};

describe("App", () => {
  beforeEach(() => {
    useAppStateMock.mockReturnValue(makeStatus() as AppStatus);
    useDisguiseStateMock.mockReturnValue(disguise);
    useUpdateCheckMock.mockReturnValue({ installing: false, updateVersion: null });
  });

  it("renders the loading state before status is available", () => {
    useAppStateMock.mockReturnValue(null);
    render(<App />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
    expect(screen.queryByTestId("status-panel")).not.toBeInTheDocument();
  });

  it("renders the loading state in light mode", () => {
    localStorage.setItem("insomniapp-theme", "light");
    useAppStateMock.mockReturnValue(null);
    render(<App />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("composes the panels once status is available", () => {
    render(<App />);
    expect(screen.getByTestId("status-panel")).toBeInTheDocument();
    expect(screen.getByTestId("idle-timer")).toBeInTheDocument();
    expect(screen.getByTestId("settings-form")).toBeInTheDocument();
    expect(screen.queryByText(/Updating to/)).not.toBeInTheDocument();
  });

  it("composes the panels in light mode", () => {
    localStorage.setItem("insomniapp-theme", "light");
    render(<App />);
    expect(screen.getByTestId("status-panel")).toBeInTheDocument();
  });

  it("shows the update banner with the attempted version", () => {
    useUpdateCheckMock.mockReturnValue({ installing: true, updateVersion: "2.0.0" });
    render(<App />);
    expect(screen.getByText("Updating to v2.0.0...")).toBeInTheDocument();
  });

  it("falls back to 'latest' in the update banner when no version is known", () => {
    useUpdateCheckMock.mockReturnValue({ installing: true, updateVersion: null });
    render(<App />);
    expect(screen.getByText("Updating to vlatest...")).toBeInTheDocument();
  });
});
