import { describe, expect, it } from "vitest";

import {
  TRAY_PANEL_DEFAULT_MAX_HEIGHT_PX,
  TRAY_PANEL_MIN_HEIGHT_PX,
  TRAY_PANEL_SHELL_CHROME_PX,
  TRAY_PANEL_VIEWPORT_MARGIN_PX,
  TRAY_PANEL_WIDTH_PX,
  clampTrayPanelHeight,
  measureTrayPanelLayoutHeight,
  sumTrayPanelBlockHeights,
  trayPanelMaxHeightPx,
  trayPanelScrollRegionClassName,
  trayPanelShellClassName,
} from "./tray-panel-layout";

describe("trayPanelLayout", () => {
  it("defines tray panel dimensions aligned with the Tauri main window", () => {
    expect(TRAY_PANEL_WIDTH_PX).toBe(360);
    expect(TRAY_PANEL_MIN_HEIGHT_PX).toBeGreaterThan(0);
    expect(TRAY_PANEL_DEFAULT_MAX_HEIGHT_PX).toBe(480);
  });

  it("caps panel height to viewport minus margin", () => {
    expect(trayPanelMaxHeightPx(900)).toBe(900 - TRAY_PANEL_VIEWPORT_MARGIN_PX);
  });

  it("clamps content height between min and viewport max", () => {
    expect(clampTrayPanelHeight(80, 900)).toBe(TRAY_PANEL_MIN_HEIGHT_PX);
    expect(clampTrayPanelHeight(320, 900)).toBe(320);
    expect(clampTrayPanelHeight(2000, 900)).toBe(trayPanelMaxHeightPx(900));
  });

  it("sums scroll content, separator, and footer block heights", () => {
    expect(
      sumTrayPanelBlockHeights({
        contentScrollHeight: 280,
        separatorHeight: 1,
        footerHeight: 168,
      }),
    ).toBe(449);
  });

  it("includes shell top padding in measured tray height", () => {
    expect(measureTrayPanelLayoutHeight(null)).toBe(0);
    expect(TRAY_PANEL_SHELL_CHROME_PX).toBe(12);
  });

  it("keeps the shell clipped with fully rounded corners", () => {
    const shell = trayPanelShellClassName();
    expect(shell).toContain("overflow-hidden");
    expect(shell).not.toContain("overflow-x-hidden");
    expect(shell).toContain("rounded-[var(--radius-tray-panel)]");
    expect(shell).toContain("h-full");
    expect(shell).toContain("flex-1");
    expect(shell).not.toContain("h-auto");
    expect(shell).not.toContain("ring-1");
    expect(shell).not.toContain("shadow-sm");
    expect(shell).toContain("tray-panel");
    expect(shell).toContain("pt-3");
  });

  it("sizes the scroll region as a flex child inside the rounded shell", () => {
    const scrollRegion = trayPanelScrollRegionClassName();
    expect(scrollRegion).toContain("min-h-0");
    expect(scrollRegion).toContain("flex-1");
  });
});
