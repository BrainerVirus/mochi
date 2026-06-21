// @vitest-environment happy-dom

import { cleanup, fireEvent, render, renderHook, screen } from "@testing-library/react";
import { createElement, useRef, type ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAppSegmentControlState } from "@/components/ui/use-app-segment-control-state";
import { TrayPanelTabList } from "@/features/tray/components/tray-panel-tab-list";
import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

const indicatorMocks = vi.hoisted(() => ({
  executeCommand: vi.fn<(command: unknown, context: unknown) => void>(),
}));

vi.mock("@/features/tray/components/tray-segment-indicator", async (importOriginal) => ({
  ...(await importOriginal()),
  executeTraySegmentIndicatorCommand: indicatorMocks.executeCommand,
}));

vi.mock("@/features/tray/components/tray-segmented-control", async () => {
  const { AppSegmentedControl, AppSegmentedControlView } =
    await import("@/components/ui/app-segmented-control");

  return {
    TraySegmentedControl: ({
      tabs,
      value,
      onValueChange,
    }: {
      tabs: TrayPanelTab[];
      value: string;
      onValueChange: (value: string) => void;
    }) =>
      createElement(AppSegmentedControl, {
        items: tabs,
        value,
        onValueChange,
        stretchItems: false,
      }),
    TraySegmentedControlView: ({
      tabs,
      value,
      state,
    }: {
      tabs: TrayPanelTab[];
      value: string;
      state: ReturnType<typeof useAppSegmentControlState>;
    }) =>
      createElement(AppSegmentedControlView, {
        items: tabs,
        value,
        state,
        stretchItems: false,
      }),
  };
});

vi.mock("@/features/tray/components/scroll-fade-region", () => ({
  ScrollFadeRegion: ({
    children,
    onCycleForward,
    onCycleBackward,
  }: {
    children: ReactNode;
    onCycleForward?: (scrollEl: HTMLDivElement) => void;
    onCycleBackward?: (scrollEl: HTMLDivElement) => void;
  }) => {
    const scrollRef = useRef<HTMLDivElement>(null);

    return createElement(
      "div",
      null,
      createElement(
        "button",
        {
          type: "button",
          "aria-label": "Cycle backward",
          onClick: () => scrollRef.current && onCycleBackward?.(scrollRef.current),
        },
        "Back",
      ),
      createElement("div", { ref: scrollRef }, children),
      createElement(
        "button",
        {
          type: "button",
          "aria-label": "Cycle forward",
          onClick: () => scrollRef.current && onCycleForward?.(scrollRef.current),
        },
        "Next",
      ),
    );
  },
}));

const tabs = [
  { id: "overview", label: "Overview" },
  { id: "codex", label: "Codex" },
] satisfies TrayPanelTab[];

afterEach(cleanup);

function renderTabList(value: string, onValueChange = vi.fn<(value: string) => void>()) {
  render(createElement(TrayPanelTabList, { tabs, value, onValueChange }));
  return onValueChange;
}

function expectMoveActive(tabId: string) {
  expect(indicatorMocks.executeCommand).toHaveBeenCalledWith(
    { type: "moveActive", tabId },
    expect.any(Object),
  );
}

describe("TrayPanelTabList selection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: vi.fn<() => void>(),
    });
  });

  it("routes direct tab selection through the indicator machine", () => {
    const onValueChange = renderTabList("overview");

    fireEvent.click(screen.getByLabelText("Codex"));

    expectMoveActive("codex");
    expect(onValueChange).toHaveBeenCalledWith("codex");
  });

  it.each([
    ["forward", "overview", "Cycle forward", "codex"],
    ["backward", "codex", "Cycle backward", "overview"],
  ])("routes %s chevron selection through the indicator machine", (_, value, label, next) => {
    const onValueChange = renderTabList(value);

    fireEvent.click(screen.getByLabelText(label));

    expectMoveActive(next);
    expect(onValueChange).toHaveBeenCalledWith(next);
  });

  it("handles rapid repeated chevron selections without throwing", () => {
    const onValueChange = renderTabList("overview");
    const cycleForward = screen.getByLabelText("Cycle forward");

    expect(() => {
      fireEvent.click(cycleForward);
      fireEvent.click(cycleForward);
      fireEvent.click(cycleForward);
    }).not.toThrow();

    expect(indicatorMocks.executeCommand).toHaveBeenCalledTimes(3);
    expect(onValueChange).toHaveBeenCalledTimes(3);
  });

  it("preserves the tab group and accessible tab labels", () => {
    renderTabList("overview");

    expect(screen.getByRole("group")).toBeTruthy();
    expect(screen.getByRole("radio", { name: "Overview" }).getAttribute("aria-checked")).toBe(
      "true",
    );
    expect(screen.getByRole("radio", { name: "Codex" }).getAttribute("aria-checked")).toBe("false");
  });
});

describe("useAppSegmentControlState", () => {
  it("keeps the machine-aware selection callback stable when dependencies do not change", () => {
    const onValueChange = vi.fn<(value: string) => void>();
    const { result, rerender } = renderHook(() =>
      useAppSegmentControlState("overview", 0, onValueChange, {
        enabled: true,
        showHover: true,
        contentReady: true,
      }),
    );
    const initialHandler = result.current.handleValueChange;

    rerender();

    expect(result.current.handleValueChange).toBe(initialHandler);
  });
});
