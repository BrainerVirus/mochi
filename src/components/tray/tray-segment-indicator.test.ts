import { describe, expect, it } from "vitest";

import {
  mergeHoverIntoActiveStart,
  metricsFromClientRects,
  shouldAnimateActiveIndicator,
} from "@/components/tray/tray-segment-indicator";

describe("metricsFromClientRects", () => {
  it("returns position relative to the track using layout boxes", () => {
    expect(metricsFromClientRects({ left: 10 }, { left: 90, width: 60 })).toEqual({
      x: 80,
      width: 60,
    });
  });
});

describe("shouldAnimateActiveIndicator", () => {
  const prev = { x: 40, width: 68 };

  it("snaps on first layout without a previous position", () => {
    expect(shouldAnimateActiveIndicator(null, true, false)).toBe(false);
  });

  it("morphs when a previous position exists and animation is enabled", () => {
    expect(shouldAnimateActiveIndicator(prev, true, false)).toBe(true);
  });

  it("snaps when animation is disabled", () => {
    expect(shouldAnimateActiveIndicator(prev, false, false)).toBe(false);
  });
});

describe("mergeHoverIntoActiveStart", () => {
  it("returns null when the hovered tab does not match the selection", () => {
    expect(mergeHoverIntoActiveStart({} as HTMLElement, "alpha", "beta")).toBeNull();
  });
});
