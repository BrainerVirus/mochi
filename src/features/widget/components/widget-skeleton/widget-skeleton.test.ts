// @vitest-environment happy-dom

import { cleanup, render, screen } from "@testing-library/react";
import { createElement } from "react";
import { afterEach, expect, it } from "vitest";

import { WidgetSkeleton } from "./widget-skeleton";

afterEach(cleanup);

it("renders one widget-shaped skeleton section per configured provider", () => {
  render(createElement(WidgetSkeleton, { providerCount: 3 }));
  expect(screen.getAllByTestId("widget-skeleton-section")).toHaveLength(3);
});

it("renders a single fallback section for zero providers while pending", () => {
  render(createElement(WidgetSkeleton, { providerCount: 0 }));
  expect(screen.getAllByTestId("widget-skeleton-section")).toHaveLength(1);
});

it("disables shimmer under reduced motion", () => {
  const { container } = render(createElement(WidgetSkeleton, { providerCount: 2 }));
  const animated = container.querySelectorAll('[data-slot="skeleton"]');
  expect(animated.length).toBeGreaterThan(0);
  for (const node of animated) {
    expect(node.getAttribute("class") ?? "").toContain("motion-reduce:animate-none");
  }
});

it("announces loading to screen readers", () => {
  render(createElement(WidgetSkeleton, { providerCount: 2 }));
  expect(screen.getByRole("status").textContent).toContain("Loading provider usage");
});
