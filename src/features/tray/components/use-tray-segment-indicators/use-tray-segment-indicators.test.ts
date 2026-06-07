import { describe, expect, it } from "vitest";

import {
  shouldAnimateActiveOnValueChange,
  shouldRunMachineSelectOnValueChange,
  shouldSyncActiveOnValueChange,
} from "@/features/tray/components/use-tray-segment-indicators";

describe("shouldRunMachineSelectOnValueChange", () => {
  it("runs machine SELECT for tray page tabs so hover can hand off to moveActive", () => {
    expect(shouldRunMachineSelectOnValueChange({ showHover: true })).toBe(true);
    expect(shouldRunMachineSelectOnValueChange({})).toBe(true);
  });

  it("skips machine SELECT for settings page tabs so layout sync tweens once", () => {
    expect(shouldRunMachineSelectOnValueChange({ showHover: false })).toBe(false);
  });

  it("skips machine SELECT for inline controls so only layout sync tweens once", () => {
    expect(shouldRunMachineSelectOnValueChange({ showHover: false })).toBe(false);
  });
});

describe("shouldSyncActiveOnValueChange", () => {
  it("defers value-change sync to machine SELECT for tray page tabs", () => {
    expect(shouldSyncActiveOnValueChange({ showHover: true })).toBe(false);
  });

  it("runs layout sync on value change for settings and inline controls", () => {
    expect(shouldSyncActiveOnValueChange({ showHover: false })).toBe(true);
  });
});

describe("shouldAnimateActiveOnValueChange", () => {
  it("animates only after initial placement and content are ready", () => {
    expect(shouldAnimateActiveOnValueChange(false, true)).toBe(false);
    expect(shouldAnimateActiveOnValueChange(true, false)).toBe(false);
    expect(shouldAnimateActiveOnValueChange(true, true)).toBe(true);
  });
});
