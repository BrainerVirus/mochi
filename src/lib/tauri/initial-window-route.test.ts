import { describe, expect, it } from "vitest";

import { APP_WINDOW_LABEL } from "@/lib/tauri/app-window";
import {
  initialRouteForWindowLabel,
  shouldNavigateFromPackagedShell,
  WIDGET_WINDOW_LABEL,
} from "@/lib/tauri/initial-window-route";
import { TRAY_PANEL_WINDOW_LABEL } from "@/lib/tauri/tray-panel-window";

describe("initialRouteForWindowLabel", () => {
  it("maps known window labels to client routes", () => {
    expect(initialRouteForWindowLabel(TRAY_PANEL_WINDOW_LABEL)).toBe("/");
    expect(initialRouteForWindowLabel(APP_WINDOW_LABEL)).toBe("/settings");
    expect(initialRouteForWindowLabel(WIDGET_WINDOW_LABEL)).toBe("/widget");
  });

  it("defaults unknown labels to the tray route", () => {
    expect(initialRouteForWindowLabel("unknown")).toBe("/");
  });
});

describe("shouldNavigateFromPackagedShell", () => {
  it("detects shell boot paths", () => {
    expect(shouldNavigateFromPackagedShell("/")).toBe(true);
    expect(shouldNavigateFromPackagedShell("/index.html")).toBe(true);
    expect(shouldNavigateFromPackagedShell("/_shell.html")).toBe(true);
  });

  it("ignores deep routes already resolved by the router", () => {
    expect(shouldNavigateFromPackagedShell("/settings")).toBe(false);
    expect(shouldNavigateFromPackagedShell("/widget")).toBe(false);
  });
});
