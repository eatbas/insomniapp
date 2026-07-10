import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, act, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import DisguiseWindow from "./DisguiseWindow";
import { renderWithTheme } from "../test/utils";
import type { DisguiseState } from "../types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { relaunch } from "@tauri-apps/plugin-process";

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);
const relaunchMock = vi.mocked(relaunch);

const SUPPORTED: DisguiseState = {
  supported: true,
  currentName: "insomniAPP",
  isDisguised: false,
};

const backend = {
  disguise: SUPPORTED as DisguiseState,
  apps: [] as string[],
  disguiseRejects: false,
  applyRejects: false,
  resetRejects: false,
};

let refreshHandler: (() => void) | undefined;
const unlistenFn = vi.fn();

function configure(overrides: Partial<typeof backend>) {
  Object.assign(backend, overrides);
}

describe("DisguiseWindow", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
    relaunchMock.mockReset();
    unlistenFn.mockReset();
    refreshHandler = undefined;
    Object.assign(backend, {
      disguise: SUPPORTED,
      apps: [],
      disguiseRejects: false,
      applyRejects: false,
      resetRejects: false,
    });

    invokeMock.mockImplementation((cmd: string) => {
      switch (cmd) {
        case "get_disguise_state":
          return backend.disguiseRejects
            ? Promise.reject(new Error("state failed"))
            : Promise.resolve(backend.disguise);
        case "list_running_apps":
          return Promise.resolve(backend.apps);
        case "apply_disguise":
          return backend.applyRejects
            ? Promise.reject(new Error("apply failed"))
            : Promise.resolve();
        case "reset_disguise":
          return backend.resetRejects
            ? Promise.reject(new Error("reset failed"))
            : Promise.resolve();
        default:
          return Promise.resolve();
      }
    });

    listenMock.mockImplementation((_event: string, handler: unknown) => {
      refreshHandler = handler as () => void;
      return Promise.resolve(unlistenFn);
    });
    relaunchMock.mockResolvedValue(undefined as never);
  });

  it("shows a loading state, then the running apps and selects the first", async () => {
    configure({ apps: ["Slack", "Chrome"] });
    renderWithTheme(<DisguiseWindow />, "dark");

    expect(screen.getByText("Loading running apps...")).toBeInTheDocument();

    expect(await screen.findByRole("button", { name: "Slack" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Chrome" })).toBeInTheDocument();
    expect(screen.getByText("insomniAPP")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Apply & Restart" })).toBeEnabled();
  });

  it("shows an empty state and disables Apply when no apps are visible", async () => {
    configure({ apps: [] });
    renderWithTheme(<DisguiseWindow />);

    expect(
      await screen.findByText("No visible apps found. Open an app and click Refresh."),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Apply & Restart" })).toBeDisabled();
  });

  it("renders the unsupported message on non-Windows platforms", async () => {
    configure({ disguise: { supported: false, currentName: "insomniAPP", isDisguised: false } });
    renderWithTheme(<DisguiseWindow />);

    expect(
      await screen.findByText("Disguise mode is not available on this OS."),
    ).toBeInTheDocument();
  });

  it("surfaces a load error when the backend rejects", async () => {
    configure({ disguiseRejects: true });
    renderWithTheme(<DisguiseWindow />);

    expect(
      await screen.findByText("Failed to load running apps. Please try Refresh."),
    ).toBeInTheDocument();
  });

  it("reloads when a refresh-apps event arrives and keeps a still-present selection", async () => {
    configure({ apps: ["Slack"] });
    renderWithTheme(<DisguiseWindow />);
    await screen.findByRole("button", { name: "Slack" });

    configure({ apps: ["Slack", "Zoom"] });
    await act(async () => {
      refreshHandler?.();
    });

    expect(await screen.findByRole("button", { name: "Zoom" })).toBeInTheDocument();
    // "Slack" was already selected and remains in the list, so it stays selected.
    expect(screen.getByRole("button", { name: "Slack" })).toHaveClass("bg-indigo-600");
  });

  it("reloads when the Refresh button is clicked", async () => {
    const user = userEvent.setup();
    configure({ apps: ["Slack"] });
    renderWithTheme(<DisguiseWindow />);
    await screen.findByRole("button", { name: "Slack" });

    const before = invokeMock.mock.calls.filter(([c]) => c === "get_disguise_state").length;
    await user.click(screen.getByRole("button", { name: "Refresh" }));

    await waitFor(() => {
      const after = invokeMock.mock.calls.filter(([c]) => c === "get_disguise_state").length;
      expect(after).toBeGreaterThan(before);
    });
  });

  it("changes the selection when another app is clicked (light mode)", async () => {
    const user = userEvent.setup();
    configure({ apps: ["Slack", "Chrome"] });
    renderWithTheme(<DisguiseWindow />, "light");
    await screen.findByRole("button", { name: "Slack" });

    await user.click(screen.getByRole("button", { name: "Chrome" }));
    expect(screen.getByRole("button", { name: "Chrome" })).toHaveClass("bg-indigo-500");
    expect(screen.getByRole("button", { name: "Slack" })).not.toHaveClass("bg-indigo-500");
  });

  it("applies the disguise and relaunches on success", async () => {
    const user = userEvent.setup();
    configure({ apps: ["Slack"] });
    renderWithTheme(<DisguiseWindow />);
    await screen.findByRole("button", { name: "Slack" });

    await user.click(screen.getByRole("button", { name: "Apply & Restart" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("apply_disguise", { name: "Slack" }),
    );
    expect(relaunchMock).toHaveBeenCalled();
  });

  it("shows an error and returns to idle when apply fails", async () => {
    const user = userEvent.setup();
    configure({ apps: ["Slack"], applyRejects: true });
    renderWithTheme(<DisguiseWindow />);
    await screen.findByRole("button", { name: "Slack" });

    await user.click(screen.getByRole("button", { name: "Apply & Restart" }));

    expect(await screen.findByText("Failed to apply disguise name.")).toBeInTheDocument();
    expect(relaunchMock).not.toHaveBeenCalled();
  });

  it("resets the disguise and relaunches on success", async () => {
    const user = userEvent.setup();
    configure({ apps: ["Slack"] });
    renderWithTheme(<DisguiseWindow />);
    await screen.findByRole("button", { name: "Slack" });

    await user.click(screen.getByRole("button", { name: "Reset to insomniAPP" }));

    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("reset_disguise"));
    expect(relaunchMock).toHaveBeenCalled();
  });

  it("shows an error and returns to idle when reset fails", async () => {
    const user = userEvent.setup();
    configure({ apps: ["Slack"], resetRejects: true });
    renderWithTheme(<DisguiseWindow />);
    await screen.findByRole("button", { name: "Slack" });

    await user.click(screen.getByRole("button", { name: "Reset to insomniAPP" }));

    expect(await screen.findByText("Failed to reset disguise name.")).toBeInTheDocument();
    expect(relaunchMock).not.toHaveBeenCalled();
  });

  it("unsubscribes from the refresh event on unmount", async () => {
    configure({ apps: ["Slack"] });
    const { unmount } = renderWithTheme(<DisguiseWindow />);
    await screen.findByRole("button", { name: "Slack" });

    const before = unlistenFn.mock.calls.length;
    unmount();
    await act(async () => {});
    expect(unlistenFn.mock.calls.length).toBe(before + 1);
  });
});
