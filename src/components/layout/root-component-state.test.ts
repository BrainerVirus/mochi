import { describe, expect, it } from "vitest";

import {
  getHydrationSafeRootState,
  shouldUseFullHeightWindowShell,
} from "@/components/layout/root-component-state";

describe("getHydrationSafeRootState", () => {
  it("uses server-safe defaults so Tauri-only window state is applied after hydration", () => {
    expect(getHydrationSafeRootState()).toEqual({
      isTrayPanelWindow: false,
      isAppWindow: false,
      isWidgetWindow: false,
      platform: "unknown",
    });
  });
});

describe("shouldUseFullHeightWindowShell", () => {
  it("uses full-height shell classes for linux app and tray windows", () => {
    expect(
      shouldUseFullHeightWindowShell({
        platform: "linux",
        isTrayPanelWindow: true,
        isAppWindow: false,
      }),
    ).toBe(true);
    expect(
      shouldUseFullHeightWindowShell({
        platform: "linux",
        isTrayPanelWindow: false,
        isAppWindow: true,
      }),
    ).toBe(true);
  });

  it("uses full-height shell classes for linux widget windows", () => {
    expect(
      shouldUseFullHeightWindowShell({
        platform: "linux",
        isTrayPanelWindow: false,
        isAppWindow: false,
        isWidgetWindow: true,
      }),
    ).toBe(true);
  });
});
