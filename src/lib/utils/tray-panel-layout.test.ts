import { describe, expect, it } from "vitest";

import {
  TRAY_PANEL_MAX_HEIGHT_PX,
  TRAY_PANEL_WIDTH_PX,
  trayPanelScrollRegionClassName,
  trayPanelShellClassName,
} from "./tray-panel-layout";

describe("trayPanelLayout", () => {
  it("defines tray panel dimensions aligned with the Tauri main window", () => {
    expect(TRAY_PANEL_WIDTH_PX).toBe(360);
    expect(TRAY_PANEL_MAX_HEIGHT_PX).toBe(480);
  });

  it("keeps the shell clipped with fully rounded corners", () => {
    const shell = trayPanelShellClassName();
    expect(shell).toContain("overflow-hidden");
    expect(shell).toContain("rounded-mochi");
    expect(shell).toContain("h-full");
    expect(shell).toContain("tray-panel");
  });

  it("enables vertical scrolling inside the rounded shell", () => {
    const scrollRegion = trayPanelScrollRegionClassName();
    expect(scrollRegion).toContain("overflow-y-auto");
    expect(scrollRegion).toContain("min-h-0");
  });
});
