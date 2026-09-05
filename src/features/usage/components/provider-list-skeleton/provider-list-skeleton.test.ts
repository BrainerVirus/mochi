// @vitest-environment happy-dom

import { render, screen } from "@testing-library/react";
import { createElement } from "react";
import { expect, it } from "vitest";

import { ProviderListSkeleton } from "./provider-list-skeleton";

it("renders one skeleton row per configured provider", () => {
  render(createElement(ProviderListSkeleton, { providerCount: 3 }));
  expect(screen.getAllByTestId("provider-skeleton-row")).toHaveLength(3);
});

it("renders nothing for zero providers", () => {
  const { container } = render(createElement(ProviderListSkeleton, { providerCount: 0 }));
  expect(container.firstChild).toBeNull();
});
