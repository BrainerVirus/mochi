// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from "vitest";

import { cycleTrayPanelTabs } from "@/features/tray/components/tray-panel-tab-cycle";
import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

const scrollMocks = vi.hoisted(() => ({
  backward: vi.fn<(scrollEl: HTMLDivElement, fadeInset: number) => void>(),
  forward: vi.fn<(scrollEl: HTMLDivElement, fadeInset: number) => void>(),
}));

vi.mock("@/features/tray/components/scroll-fade-cycle", () => ({
  cycleHorizontalScrollBackward: scrollMocks.backward,
  cycleHorizontalScrollForward: scrollMocks.forward,
}));

const tabs = [
  { id: "overview", label: "Overview" },
  { id: "codex", label: "Codex" },
] satisfies TrayPanelTab[];

describe("cycleTrayPanelTabs", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("ignores a current value that is no longer in the tab list", () => {
    const scrollEl = document.createElement("div");
    const onValueChange = vi.fn<(value: string) => void>();

    cycleTrayPanelTabs(scrollEl, tabs, "cursor", "forward", onValueChange, 40);

    expect(onValueChange).not.toHaveBeenCalled();
    expect(scrollMocks.forward).not.toHaveBeenCalled();
    expect(scrollMocks.backward).not.toHaveBeenCalled();
  });

  it.each([
    ["forward", "codex", scrollMocks.forward],
    ["backward", "overview", scrollMocks.backward],
  ] as const)(
    "falls back to %s scrolling at the clamped tab-list end",
    (direction, value, scroll) => {
      const scrollEl = document.createElement("div");
      const onValueChange = vi.fn<(value: string) => void>();

      cycleTrayPanelTabs(scrollEl, tabs, value, direction, onValueChange, 40);

      expect(onValueChange).not.toHaveBeenCalled();
      expect(scroll).toHaveBeenCalledWith(scrollEl, 40);
    },
  );
});
