import { describe, expect, it } from "vitest";

import {
  TRAY_PANEL_HEIGHT_DURATION_S,
  TRAY_PANEL_HEIGHT_EASE,
} from "./tray-panel-height-animation";

describe("trayPanelHeightAnimation", () => {
  it("uses smooth tab morph timing defaults", () => {
    expect(TRAY_PANEL_HEIGHT_DURATION_S).toBeGreaterThanOrEqual(0.25);
    expect(TRAY_PANEL_HEIGHT_DURATION_S).toBeLessThanOrEqual(0.4);
    expect(TRAY_PANEL_HEIGHT_EASE).toBe("power2.out");
  });
});
