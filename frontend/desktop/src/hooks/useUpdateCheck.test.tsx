import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { useUpdateCheck } from "./useUpdateCheck";

const h = vi.hoisted(() => ({
  checkMock: vi.fn(),
  relaunchMock: vi.fn(),
  onFocusChangedMock: vi.fn(),
  unlistenFn: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({ check: h.checkMock, Update: class {} }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: h.relaunchMock }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ onFocusChanged: h.onFocusChangedMock }),
}));

const COOLDOWN = 5 * 60 * 1000;
const INTERVAL = 4 * 60 * 60 * 1000;

let focusHandler: ((event: { payload: boolean }) => void) | undefined;

function Probe() {
  const { installing, updateVersion } = useUpdateCheck();
  return (
    <div>
      <span data-testid="installing">{String(installing)}</span>
      <span data-testid="version">{updateVersion ?? "none"}</span>
    </div>
  );
}

async function flush(times = 6) {
  for (let i = 0; i < times; i++) {
    // eslint-disable-next-line no-await-in-loop
    await act(async () => {
      await Promise.resolve();
    });
  }
}

async function fireFocus(focused: boolean) {
  await act(async () => {
    focusHandler?.({ payload: focused });
  });
}

function advance(ms: number) {
  act(() => {
    vi.advanceTimersByTime(ms);
  });
}

function availableUpdate(version: string, downloadAndInstall: () => Promise<void>) {
  return { available: true, version, downloadAndInstall };
}

describe("useUpdateCheck", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    h.checkMock.mockReset();
    h.relaunchMock.mockReset().mockResolvedValue(undefined);
    h.unlistenFn.mockReset();
    focusHandler = undefined;
    h.onFocusChangedMock.mockReset().mockImplementation((cb: unknown) => {
      focusHandler = cb as (event: { payload: boolean }) => void;
      return Promise.resolve(h.unlistenFn);
    });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("installs an available update and relaunches", async () => {
    const dl = vi.fn().mockResolvedValue(undefined);
    h.checkMock.mockResolvedValue(availableUpdate("2.0.0", dl));

    render(<Probe />);
    await flush();

    expect(screen.getByTestId("installing")).toHaveTextContent("true");
    expect(screen.getByTestId("version")).toHaveTextContent("2.0.0");
    expect(dl).toHaveBeenCalledTimes(1);
    expect(h.relaunchMock).toHaveBeenCalledTimes(1);
  });

  it("does nothing when no update is available", async () => {
    h.checkMock.mockResolvedValue(undefined);

    render(<Probe />);
    await flush();

    expect(screen.getByTestId("installing")).toHaveTextContent("false");
    expect(screen.getByTestId("version")).toHaveTextContent("none");
  });

  it("resets the installing flag when the download fails", async () => {
    const dl = vi.fn().mockRejectedValue(new Error("download failed"));
    h.checkMock.mockResolvedValue(availableUpdate("3.0.0", dl));

    render(<Probe />);
    await flush();

    expect(screen.getByTestId("installing")).toHaveTextContent("false");
    expect(h.relaunchMock).not.toHaveBeenCalled();
  });

  it("suppresses a re-check that happens within the cooldown", async () => {
    h.checkMock.mockResolvedValue(undefined);

    render(<Probe />);
    await flush();
    expect(h.checkMock).toHaveBeenCalledTimes(1);

    await fireFocus(true);
    expect(h.checkMock).toHaveBeenCalledTimes(1);
  });

  it("re-checks on focus once the cooldown has elapsed", async () => {
    h.checkMock.mockResolvedValue(undefined);

    render(<Probe />);
    await flush();

    advance(COOLDOWN + 1000);
    await fireFocus(true);
    await flush();

    expect(h.checkMock).toHaveBeenCalledTimes(2);
  });

  it("ignores focus-lost events", async () => {
    h.checkMock.mockResolvedValue(undefined);

    render(<Probe />);
    await flush();

    await fireFocus(false);
    expect(h.checkMock).toHaveBeenCalledTimes(1);
  });

  it("skips checks while an install is in progress", async () => {
    const dl = vi.fn(() => new Promise<void>(() => {}));
    h.checkMock.mockResolvedValue(availableUpdate("2.0.0", dl));

    render(<Probe />);
    await flush();
    expect(screen.getByTestId("installing")).toHaveTextContent("true");
    expect(h.checkMock).toHaveBeenCalledTimes(1);

    advance(COOLDOWN + 1000);
    await fireFocus(true);

    expect(h.checkMock).toHaveBeenCalledTimes(1);
  });

  it("swallows a rejected update check", async () => {
    h.checkMock.mockRejectedValue(new Error("network down"));

    render(<Probe />);
    await flush();

    expect(screen.getByTestId("installing")).toHaveTextContent("false");
  });

  it("re-checks when the interval fires", async () => {
    h.checkMock.mockResolvedValue(undefined);

    render(<Probe />);
    await flush();

    advance(INTERVAL);
    await flush();

    expect(h.checkMock.mock.calls.length).toBeGreaterThanOrEqual(2);
  });

  it("clears the interval and focus listener on unmount", async () => {
    h.checkMock.mockResolvedValue(undefined);

    const { unmount } = render(<Probe />);
    await flush();

    unmount();
    await flush();

    expect(h.unlistenFn).toHaveBeenCalledTimes(1);
  });

  it("does not start a second install for an update already being installed", async () => {
    const dl = vi.fn(() => new Promise<void>(() => {}));
    const update = availableUpdate("2.0.0", dl);

    let resolveFirst!: (value: typeof update) => void;
    let resolveSecond!: (value: typeof update) => void;
    h.checkMock
      .mockReturnValueOnce(new Promise((res) => (resolveFirst = res)))
      .mockReturnValueOnce(new Promise((res) => (resolveSecond = res)));

    render(<Probe />);
    await flush();

    // Second check dispatched (past the cooldown) before the first resolves, so
    // both reach installUpdate; the second must be short-circuited by the guard.
    advance(COOLDOWN + 1000);
    await fireFocus(true);

    await act(async () => {
      resolveFirst(update);
      resolveSecond(update);
    });
    await flush();

    expect(h.checkMock).toHaveBeenCalledTimes(2);
    expect(dl).toHaveBeenCalledTimes(1);
  });
});
