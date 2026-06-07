import { describe, expect, it } from "vitest";

import { isSettingsRoutePath } from "./settings-tab-state";

describe("isSettingsRoutePath", () => {
  it("matches the settings route", () => {
    expect(isSettingsRoutePath("/settings")).toBe(true);
    expect(isSettingsRoutePath("/settings/")).toBe(true);
  });

  it("rejects other app routes", () => {
    expect(isSettingsRoutePath("/about")).toBe(false);
    expect(isSettingsRoutePath("/update")).toBe(false);
    expect(isSettingsRoutePath("/")).toBe(false);
  });

  it("ignores query and hash segments", () => {
    expect(isSettingsRoutePath("/settings?tab=providers")).toBe(true);
    expect(isSettingsRoutePath("/settings#providers")).toBe(true);
  });
});
