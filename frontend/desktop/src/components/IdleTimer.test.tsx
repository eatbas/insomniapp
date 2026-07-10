import { describe, it, expect } from "vitest";
import { screen } from "@testing-library/react";
import IdleTimer from "./IdleTimer";
import { makeStatus, renderWithTheme } from "../test/utils";

function progressBar(container: HTMLElement): HTMLElement {
  const bar = container.querySelector<HTMLElement>('[style*="width"]');
  if (!bar) throw new Error("progress bar not found");
  return bar;
}

describe("IdleTimer", () => {
  it("shows a partial blue bar below the idle threshold", () => {
    const { container } = renderWithTheme(
      <IdleTimer status={makeStatus({ idleSeconds: 30, idleThresholdSecs: 60, isIdle: false })} />,
    );

    expect(screen.getByText("00:30")).toBeInTheDocument();
    expect(screen.getByText("01:00")).toBeInTheDocument();
    const bar = progressBar(container);
    expect(bar).toHaveClass("bg-blue-500");
    expect(bar.style.width).toBe("50%");
  });

  it("caps the bar at 100% and turns orange once idle past the threshold", () => {
    const { container } = renderWithTheme(
      <IdleTimer status={makeStatus({ idleSeconds: 120, idleThresholdSecs: 60, isIdle: true })} />,
    );

    const bar = progressBar(container);
    expect(bar).toHaveClass("bg-orange-500");
    expect(bar.style.width).toBe("100%");
  });

  it("reaches exactly 100% at the threshold", () => {
    const { container } = renderWithTheme(
      <IdleTimer status={makeStatus({ idleSeconds: 60, idleThresholdSecs: 60, isIdle: true })} />,
    );

    expect(progressBar(container).style.width).toBe("100%");
  });

  it("renders correctly in light mode", () => {
    renderWithTheme(
      <IdleTimer status={makeStatus({ idleSeconds: 10, idleThresholdSecs: 60 })} />,
      "light",
    );
    expect(screen.getByText("00:10")).toBeInTheDocument();
  });
});
