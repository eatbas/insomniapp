import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import StatusPanel from "./StatusPanel";
import { makeStatus, renderWithTheme } from "../test/utils";
import type { AppStatus } from "../types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";

const invokeMock = vi.mocked(invoke);

function renderPanel(
  status: AppStatus,
  { appName = "insomniAPP", disguiseSupported = false, theme = "dark" as const } = {},
) {
  return renderWithTheme(
    <StatusPanel status={status} appName={appName} disguiseSupported={disguiseSupported} />,
    theme,
  );
}

describe("StatusPanel", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it("shows the disabled state and an Enable button when turned off", () => {
    renderPanel(makeStatus({ enabled: false }), { appName: "Slack" });

    expect(screen.getByText("Disabled")).toBeInTheDocument();
    expect(screen.getByText("Slack")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Enable" })).toBeInTheDocument();
  });

  it("shows the locked state", () => {
    renderPanel(makeStatus({ enabled: true, isSessionLocked: true }));
    expect(screen.getByText("Paused (Locked)")).toBeInTheDocument();
  });

  it("shows the screen-off state", () => {
    renderPanel(makeStatus({ enabled: true, isDisplayOff: true }));
    expect(screen.getByText("Paused (Screen Off)")).toBeInTheDocument();
  });

  it("shows the active state while simulating", () => {
    renderPanel(makeStatus({ enabled: true, isSimulating: true }));
    expect(screen.getByText("Active")).toBeInTheDocument();
  });

  it("shows the monitoring state when enabled and otherwise idle", () => {
    renderPanel(makeStatus({ enabled: true }));
    expect(screen.getByText("Monitoring")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Disable" })).toBeInTheDocument();
  });

  it("invokes toggle_enabled when the enable/disable button is clicked", async () => {
    const user = userEvent.setup();
    renderPanel(makeStatus({ enabled: true }));

    await user.click(screen.getByRole("button", { name: "Disable" }));
    expect(invokeMock).toHaveBeenCalledWith("toggle_enabled");
  });

  it("toggles the theme and switches the toggle affordance", async () => {
    const user = userEvent.setup();
    renderPanel(makeStatus({ enabled: true }), { theme: "dark" });

    const toggle = screen.getByTitle("Switch to light mode");
    await user.click(toggle);
    expect(screen.getByTitle("Switch to dark mode")).toBeInTheDocument();
  });

  it("renders correctly in light mode, including the disguise affordance", () => {
    renderPanel(makeStatus({ enabled: true }), {
      theme: "light",
      disguiseSupported: true,
    });
    expect(screen.getByTitle("Switch to dark mode")).toBeInTheDocument();
    expect(screen.getByTitle("Open disguise options")).toBeInTheDocument();
  });

  it("exposes the disguise affordance only when supported", async () => {
    const user = userEvent.setup();
    const { unmount } = renderPanel(makeStatus({ enabled: true }), {
      disguiseSupported: true,
    });

    const disguise = screen.getByTitle("Open disguise options");
    await user.click(disguise);
    expect(invokeMock).toHaveBeenCalledWith("open_disguise_window");

    unmount();

    renderPanel(makeStatus({ enabled: true }), { disguiseSupported: false });
    expect(screen.queryByTitle("Open disguise options")).not.toBeInTheDocument();
  });
});
