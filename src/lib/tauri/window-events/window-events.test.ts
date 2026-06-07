import { describe, expect, it } from "vitest";

import {
  shouldHandleAppNavigateForWindow,
  shouldHandleTrayNavigateForWindow,
} from "@/lib/tauri/window-events";

describe("window event routing", () => {
  it("routes tray navigation only to the tray panel webview", () => {
    expect(shouldHandleTrayNavigateForWindow(true, false)).toBe(true);
    expect(shouldHandleAppNavigateForWindow(true, false)).toBe(false);
  });

  it("routes app navigation only to the settings webview", () => {
    expect(shouldHandleTrayNavigateForWindow(false, true)).toBe(false);
    expect(shouldHandleAppNavigateForWindow(false, true)).toBe(true);
  });
});
