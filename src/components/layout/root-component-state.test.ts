import { describe, expect, it } from "vitest";

import { getHydrationSafeRootState } from "@/components/layout/root-component-state";

describe("getHydrationSafeRootState", () => {
  it("uses server-safe defaults so Tauri-only window state is applied after hydration", () => {
    expect(getHydrationSafeRootState()).toEqual({
      isTrayPanelWindow: false,
      platform: "unknown",
    });
  });
});
