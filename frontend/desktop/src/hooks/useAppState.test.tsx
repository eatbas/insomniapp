import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { useAppState } from "./useAppState";
import { makeStatus } from "../test/utils";
import type { AppStatus } from "../types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);

const unlistenFn = vi.fn();
let eventHandler: ((event: { payload: AppStatus }) => void) | undefined;

function Probe() {
  const status = useAppState();
  return <div data-testid="status">{status ? status.idleSeconds : "none"}</div>;
}

describe("useAppState", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
    unlistenFn.mockReset();
    eventHandler = undefined;
    invokeMock.mockResolvedValue(makeStatus({ idleSeconds: 5 }));
    listenMock.mockImplementation((_event: string, handler: unknown) => {
      eventHandler = handler as (event: { payload: AppStatus }) => void;
      return Promise.resolve(unlistenFn);
    });
  });

  it("loads the initial status via get_status", async () => {
    render(<Probe />);

    expect(await screen.findByText("5")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("get_status");
    expect(listenMock).toHaveBeenCalledWith("status-update", expect.any(Function));
  });

  it("updates the status when a status-update event fires", async () => {
    render(<Probe />);
    await screen.findByText("5");

    await act(async () => {
      eventHandler?.({ payload: makeStatus({ idleSeconds: 42 }) });
    });

    expect(screen.getByTestId("status")).toHaveTextContent("42");
  });

  it("unsubscribes from the event on unmount", async () => {
    const { unmount } = render(<Probe />);
    await screen.findByText("5");

    unmount();
    await act(async () => {});

    expect(unlistenFn).toHaveBeenCalledTimes(1);
  });

  it("stays in the loading state when get_status rejects", async () => {
    invokeMock.mockRejectedValue(new Error("ipc down"));
    render(<Probe />);

    await act(async () => {});

    expect(screen.getByTestId("status")).toHaveTextContent("none");
  });
});
