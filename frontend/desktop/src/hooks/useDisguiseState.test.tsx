import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { useDisguiseState } from "./useDisguiseState";
import type { DisguiseState } from "../types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";

const invokeMock = vi.mocked(invoke);

function Probe() {
  const state = useDisguiseState();
  return (
    <div>
      <span data-testid="name">{state.currentName}</span>
      <span data-testid="supported">{String(state.supported)}</span>
    </div>
  );
}

describe("useDisguiseState", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("returns the resolved disguise state", async () => {
    const state: DisguiseState = {
      supported: true,
      currentName: "Slack",
      isDisguised: true,
    };
    invokeMock.mockResolvedValue(state);

    render(<Probe />);

    expect(await screen.findByText("Slack")).toBeInTheDocument();
    expect(screen.getByTestId("supported")).toHaveTextContent("true");
    expect(invokeMock).toHaveBeenCalledWith("get_disguise_state");
  });

  it("falls back to the default state when the command fails", async () => {
    invokeMock.mockRejectedValue(new Error("unsupported"));

    render(<Probe />);

    await act(async () => {});

    expect(screen.getByTestId("name")).toHaveTextContent("insomniAPP");
    expect(screen.getByTestId("supported")).toHaveTextContent("false");
  });
});
