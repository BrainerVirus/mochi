// @vitest-environment happy-dom

import { cleanup, render, screen } from "@testing-library/react";
import { createElement } from "react";
import { afterEach, expect, it } from "vitest";

import { TrayPanelSkeleton } from "./tray-panel-skeleton";

afterEach(cleanup);

it("renders one tray-shaped skeleton row per configured provider", () => {
  render(createElement(TrayPanelSkeleton, { providerCount: 3 }));
  expect(screen.getAllByTestId("tray-panel-skeleton-row")).toHaveLength(3);
});

it("renders a single fallback row for zero providers while pending", () => {
  render(createElement(TrayPanelSkeleton, { providerCount: 0 }));
  expect(screen.getAllByTestId("tray-panel-skeleton-row")).toHaveLength(1);
});

it("disables shimmer under reduced motion", () => {
  const { container } = render(createElement(TrayPanelSkeleton, { providerCount: 2 }));
  const animated = container.querySelectorAll('[data-slot="skeleton"]');
  expect(animated.length).toBeGreaterThan(0);
  for (const node of animated) {
    expect(node.getAttribute("class") ?? "").toContain("motion-reduce:animate-none");
  }
});

it("announces loading to screen readers", () => {
  render(createElement(TrayPanelSkeleton, { providerCount: 2 }));
  expect(screen.getByRole("status").textContent).toContain("Loading provider usage");
});
