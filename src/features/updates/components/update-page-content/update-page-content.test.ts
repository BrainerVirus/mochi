// @vitest-environment happy-dom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { createElement } from "react";
import { afterEach, expect, it, vi } from "vitest";

import { UpdatePageContent } from "./update-page-content";
vi.mock("@/features/updates/hooks/use-update-install/use-update-install", () => ({
  useUpdateInstall: () => ({
    phase: "idle",
    progress: null,
    isPending: false,
    errorMessage: null,
    mutate: vi.fn<() => void>(),
  }),
}));

afterEach(cleanup);

const baseProps = {
  notesOnly: false,
  updateAvailable: true,
  version: "1.2.3",
  channel: "stable",
  notes: "## Highlights\n- Faster startup\n- Fixed tray flicker",
  isChecking: false,
  checkError: null,
  onRecheck: vi.fn<() => void>(),
};

it("hides release notes behind a collapsed expander by default", () => {
  render(createElement(UpdatePageContent, baseProps));

  const toggle = screen.getByRole("button", { name: "Release notes" });
  expect(toggle.getAttribute("aria-expanded")).toBe("false");
  expect(screen.queryByText("Faster startup")).toBeNull();
});

it("reveals notes in a scrollable region when the expander opens", () => {
  render(createElement(UpdatePageContent, baseProps));

  const toggle = screen.getByRole("button", { name: "Release notes" });
  fireEvent.click(toggle);

  expect(toggle.getAttribute("aria-expanded")).toBe("true");
  const note = screen.getByText("Faster startup");
  expect(note).not.toBeNull();
  expect(note.closest("div.max-h-40.overflow-y-auto")).not.toBeNull();
});

it("collapses the notes again on a second toggle", () => {
  render(createElement(UpdatePageContent, baseProps));

  const toggle = screen.getByRole("button", { name: "Release notes" });
  fireEvent.click(toggle);
  fireEvent.click(toggle);

  expect(toggle.getAttribute("aria-expanded")).toBe("false");
  expect(screen.queryByText("Faster startup")).toBeNull();
});
