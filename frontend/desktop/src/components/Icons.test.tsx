import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { Logo, SunIcon, MoonIcon } from "./Icons";

describe("Icons", () => {
  it("renders the logo svg with its gradient definitions", () => {
    const { container } = render(<Logo />);
    const svg = container.querySelector("svg");
    expect(svg).not.toBeNull();
    expect(svg).toHaveAttribute("width", "14");
    expect(container.querySelectorAll("linearGradient")).toHaveLength(3);
  });

  it("renders the sun icon", () => {
    const { container } = render(<SunIcon />);
    const svg = container.querySelector("svg");
    expect(svg).not.toBeNull();
    expect(container.querySelector("circle")).not.toBeNull();
  });

  it("renders the moon icon", () => {
    const { container } = render(<MoonIcon />);
    const svg = container.querySelector("svg");
    expect(svg).not.toBeNull();
    expect(container.querySelector("path")).not.toBeNull();
  });
});
