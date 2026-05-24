import { describe, expect, it } from "vitest";

import { shouldRunMachineSelectOnValueChange } from "@/components/tray/use-tray-segment-indicators";

describe("shouldRunMachineSelectOnValueChange", () => {
  it("runs machine SELECT for page tabs so hover can hand off to moveActive", () => {
    expect(shouldRunMachineSelectOnValueChange({ showHover: true })).toBe(true);
    expect(shouldRunMachineSelectOnValueChange({})).toBe(true);
  });

  it("skips machine SELECT for inline controls so only layout sync tweens once", () => {
    expect(shouldRunMachineSelectOnValueChange({ showHover: false })).toBe(false);
  });
});
