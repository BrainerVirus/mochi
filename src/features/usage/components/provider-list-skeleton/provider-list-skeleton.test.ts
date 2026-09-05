// @vitest-environment happy-dom

import { cleanup, render, screen } from "@testing-library/react";
import { createElement } from "react";
import { afterEach, expect, it } from "vitest";

import { ProviderListSkeleton } from "./provider-list-skeleton";

afterEach(cleanup);

it("renders one skeleton row per configured provider", () => {
  render(createElement(ProviderListSkeleton, { providerCount: 3 }));
  expect(screen.getAllByTestId("provider-skeleton-row")).toHaveLength(3);
});

it("renders a single fallback row for zero providers while pending", () => {
  render(createElement(ProviderListSkeleton, { providerCount: 0 }));
  expect(screen.getAllByTestId("provider-skeleton-row")).toHaveLength(1);
});

it("disables shimmer under reduced motion", () => {
  const { container } = render(createElement(ProviderListSkeleton, { providerCount: 2 }));
  const animated = container.querySelectorAll('[data-slot="skeleton"]');
  expect(animated.length).toBeGreaterThan(0);
  for (const node of animated) {
    expect(node.getAttribute("class") ?? "").toContain("motion-reduce:animate-none");
  }
});

it("announces loading to screen readers", () => {
  render(createElement(ProviderListSkeleton, { providerCount: 2 }));
  expect(screen.getByRole("status").textContent).toContain("Loading provider usage");
});
