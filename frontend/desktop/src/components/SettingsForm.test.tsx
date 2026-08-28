import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { screen, fireEvent, act } from "@testing-library/react";
import SettingsForm from "./SettingsForm";
import { makeStatus, renderWithTheme } from "../test/utils";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";

const invokeMock = vi.mocked(invoke);
const getVersionMock = vi.mocked(getVersion);

function idleInput(): HTMLInputElement {
  return screen.getAllByRole("spinbutton")[0] as HTMLInputElement;
}

function intervalInput(): HTMLInputElement {
  return screen.getAllByRole("spinbutton")[1] as HTMLInputElement;
}

function nudgeSelect(): HTMLSelectElement {
  // Found by its accessible name: the control has no visible label, so this is
  // also the assertion that it stays reachable to a screen reader.
  return screen.getByRole("combobox", { name: "Nudge method" }) as HTMLSelectElement;
}

describe("SettingsForm", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
    getVersionMock.mockReset();
    getVersionMock.mockResolvedValue("9.9.9");
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders the app version once resolved", async () => {
    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});
    expect(screen.getByText("v9.9.9")).toBeInTheDocument();
  });

  it("hides the version label when getVersion rejects", async () => {
    getVersionMock.mockRejectedValue(new Error("app plugin unavailable"));

    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    expect(screen.queryByText(/^v\d/)).not.toBeInTheDocument();
  });

  it("pushes a debounced update for a valid idle threshold", async () => {
    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    fireEvent.change(idleInput(), { target: { value: "45" } });
    expect(invokeMock).not.toHaveBeenCalledWith("update_settings", expect.anything());

    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(invokeMock).toHaveBeenCalledWith("update_settings", {
      settings: { idleThresholdSecs: 45 },
    });
  });

  it("pushes a debounced update for the simulation interval", async () => {
    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    fireEvent.change(intervalInput(), { target: { value: "20" } });
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(invokeMock).toHaveBeenCalledWith("update_settings", {
      settings: { simulationIntervalSecs: 20 },
    });
  });

  it("ignores non-numeric input", async () => {
    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    fireEvent.change(idleInput(), { target: { value: "abc" } });
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(invokeMock).not.toHaveBeenCalledWith("update_settings", expect.anything());
  });

  it("ignores values below the minimum", async () => {
    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    fireEvent.change(idleInput(), { target: { value: "0" } });
    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(invokeMock).not.toHaveBeenCalledWith("update_settings", expect.anything());
  });

  it("renders the nudge method the backend reports", async () => {
    renderWithTheme(<SettingsForm status={makeStatus({ nudgeMethod: "f15" })} />);
    await act(async () => {});

    expect(nudgeSelect().value).toBe("f15");
  });

  it("pushes a debounced update for the nudge method", async () => {
    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    fireEvent.change(nudgeSelect(), { target: { value: "f15" } });
    expect(invokeMock).not.toHaveBeenCalledWith("update_settings", expect.anything());

    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(invokeMock).toHaveBeenCalledWith("update_settings", {
      settings: { nudgeMethod: "f15" },
    });
  });

  it("renders correctly in light mode", async () => {
    renderWithTheme(<SettingsForm status={makeStatus()} />, "light");
    await act(async () => {});
    expect(screen.getByText("v9.9.9")).toBeInTheDocument();
  });

  it("debounces rapid edits into a single update", async () => {
    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    fireEvent.change(idleInput(), { target: { value: "40" } });
    fireEvent.change(idleInput(), { target: { value: "55" } });
    act(() => {
      vi.advanceTimersByTime(500);
    });

    const updateCalls = invokeMock.mock.calls.filter(([cmd]) => cmd === "update_settings");
    expect(updateCalls).toHaveLength(1);
    expect(updateCalls[0][1]).toEqual({ settings: { idleThresholdSecs: 55 } });
  });

  it("cancels a pending update when unmounted before the debounce fires", async () => {
    const { unmount } = renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    fireEvent.change(idleInput(), { target: { value: "45" } });
    unmount();

    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(invokeMock).not.toHaveBeenCalledWith("update_settings", expect.anything());
  });

  // Vitest fails the run on an unhandled rejection, so reaching the end of this
  // test is itself the assertion that `invoke` has a rejection handler.
  it("swallows a rejected update rather than leaving an unhandled promise", async () => {
    invokeMock.mockRejectedValue(new Error("command failed"));

    renderWithTheme(<SettingsForm status={makeStatus()} />);
    await act(async () => {});

    fireEvent.change(idleInput(), { target: { value: "45" } });
    await act(async () => {
      vi.advanceTimersByTime(500);
    });

    expect(invokeMock).toHaveBeenCalledWith("update_settings", {
      settings: { idleThresholdSecs: 45 },
    });
  });
});
