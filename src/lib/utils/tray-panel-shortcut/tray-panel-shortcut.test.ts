import { describe, expect, it, vi } from "vitest";

import { trayPanelShortcut } from "./tray-panel-shortcut";

describe("trayPanelShortcut", () => {
  it("uses Command symbol on macOS", () => {
    vi.stubGlobal("navigator", { platform: "MacIntel", userAgent: "" });
    expect(trayPanelShortcut("R")).toBe("⌘R");
    vi.unstubAllGlobals();
  });

  it("uses Ctrl prefix on other platforms", () => {
    vi.stubGlobal("navigator", { platform: "Win32", userAgent: "" });
    expect(trayPanelShortcut("Q")).toBe("Ctrl+Q");
    vi.unstubAllGlobals();
  });
});
