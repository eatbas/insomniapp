import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import type { ReactElement } from "react";

const { createRootSpy, renderSpy, getCurrentWebviewWindowMock } = vi.hoisted(() => {
  const renderSpy = vi.fn();
  return {
    renderSpy,
    createRootSpy: vi.fn(() => ({ render: renderSpy })),
    getCurrentWebviewWindowMock: vi.fn(),
  };
});

vi.mock("react-dom/client", () => ({ default: { createRoot: createRootSpy } }));
vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: getCurrentWebviewWindowMock,
}));
vi.mock("./App", () => ({ default: () => <div data-testid="app-root" /> }));
vi.mock("./components/DisguiseWindow", () => ({
  default: () => <div data-testid="disguise-root" />,
}));

function renderBootstrappedTree() {
  const element = renderSpy.mock.calls[0][0] as ReactElement;
  render(element);
}

describe("main entrypoint", () => {
  beforeEach(() => {
    vi.resetModules();
    createRootSpy.mockClear();
    renderSpy.mockClear();
    getCurrentWebviewWindowMock.mockReset();
    document.body.innerHTML = '<div id="root"></div>';
  });

  it("mounts the disguise window when running in the disguise webview", async () => {
    getCurrentWebviewWindowMock.mockReturnValue({ label: "disguise" });

    await import("./main");

    expect(createRootSpy).toHaveBeenCalledWith(document.getElementById("root"));
    renderBootstrappedTree();
    expect(screen.getByTestId("disguise-root")).toBeInTheDocument();
  });

  it("mounts the main app for any other window label", async () => {
    getCurrentWebviewWindowMock.mockReturnValue({ label: "main" });

    await import("./main");

    renderBootstrappedTree();
    expect(screen.getByTestId("app-root")).toBeInTheDocument();
  });

  it("falls back to the main app when the window label cannot be read", async () => {
    getCurrentWebviewWindowMock.mockImplementation(() => {
      throw new Error("no webview context");
    });

    await import("./main");

    renderBootstrappedTree();
    expect(screen.getByTestId("app-root")).toBeInTheDocument();
  });
});
