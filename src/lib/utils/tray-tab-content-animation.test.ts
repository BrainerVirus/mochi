import { describe, expect, it } from "vitest";

import { TRAY_PANEL_HEIGHT_DURATION_S, TRAY_PANEL_HEIGHT_EASE } from "./tray-panel-height-animation";
import {
  TRAY_TAB_CONTENT_DURATION_S,
  TRAY_TAB_CONTENT_EASE,
  TRAY_TAB_CONTENT_STAGGER_S,
  TRAY_TAB_CONTENT_Y_PX,
} from "./tray-tab-content-animation";

describe("trayTabContentAnimation", () => {
  it("uses enter timing aligned with panel height morph", () => {
    expect(TRAY_TAB_CONTENT_DURATION_S).toBeGreaterThanOrEqual(0.22);
    expect(TRAY_TAB_CONTENT_DURATION_S).toBeLessThanOrEqual(TRAY_PANEL_HEIGHT_DURATION_S);
    expect(TRAY_TAB_CONTENT_EASE).toBe(TRAY_PANEL_HEIGHT_EASE);
    expect(TRAY_TAB_CONTENT_STAGGER_S).toBeGreaterThan(0);
    expect(TRAY_TAB_CONTENT_Y_PX).toBeGreaterThan(0);
  });
});
