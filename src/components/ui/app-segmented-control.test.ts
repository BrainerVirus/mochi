import { describe, expect, it } from "vitest";

import {
  SETTINGS_PAGE_TAB_DEFAULTS,
  TRAY_PAGE_TAB_DEFAULTS,
  TRAY_SEGMENT_ROW_HEIGHT,
} from "@/components/tray/tray-segmented-control";
import {
  APP_SEGMENT_INDICATOR_RADIUS_CLASS,
  APP_SEGMENT_TRACK_RADIUS_CLASS,
  INLINE_SEGMENT_INDICATOR_RADIUS_CLASS,
  resolvePageTabRadiusClasses,
  SETTINGS_SEGMENT_INDICATOR_RADIUS_CLASS,
  SETTINGS_SEGMENT_TRACK_RADIUS_CLASS,
  usesPageTabIndicators,
  usesSegmentActiveIndicator,
  usesSegmentHoverIndicator,
} from "@/components/ui/app-segmented-control";

describe("usesPageTabIndicators", () => {
  it("enables GSAP hover/active pills for page tabs", () => {
    expect(usesPageTabIndicators("page-tabs")).toBe(true);
  });

  it("returns false for inline controls", () => {
    expect(usesPageTabIndicators("inline")).toBe(false);
  });
});

describe("usesSegmentActiveIndicator", () => {
  it("enables GSAP active pill for page tabs and inline controls", () => {
    expect(usesSegmentActiveIndicator("page-tabs")).toBe(true);
    expect(usesSegmentActiveIndicator("inline")).toBe(true);
  });
});

describe("usesSegmentHoverIndicator", () => {
  it("enables hover pill only for page tabs", () => {
    expect(usesSegmentHoverIndicator("page-tabs")).toBe(true);
    expect(usesSegmentHoverIndicator("inline")).toBe(false);
  });
});

describe("TRAY_PAGE_TAB_DEFAULTS", () => {
  it("matches tray tab strip layout", () => {
    expect(TRAY_PAGE_TAB_DEFAULTS).toEqual({
      variant: "page-tabs",
      rowHeight: TRAY_SEGMENT_ROW_HEIGHT,
      stretchItems: false,
      layout: "tray",
    });
  });
});

describe("SETTINGS_PAGE_TAB_DEFAULTS", () => {
  it("uses full-width equal segments with settings layout", () => {
    expect(SETTINGS_PAGE_TAB_DEFAULTS).toEqual({
      variant: "page-tabs",
      rowHeight: "h-9",
      stretchItems: true,
      layout: "settings",
    });
  });
});

describe("resolvePageTabRadiusClasses", () => {
  it("uses fixed pill radii for tray page tabs", () => {
    expect(resolvePageTabRadiusClasses("tray")).toEqual({
      track: APP_SEGMENT_TRACK_RADIUS_CLASS,
      indicator: APP_SEGMENT_INDICATOR_RADIUS_CLASS,
    });
    expect(APP_SEGMENT_INDICATOR_RADIUS_CLASS).toBe("rounded-app-segment-indicator");
    expect(APP_SEGMENT_TRACK_RADIUS_CLASS).toBe("rounded-app-segment-track");
  });

  it("uses .app-window --radius utilities for settings page tabs", () => {
    expect(resolvePageTabRadiusClasses("settings")).toEqual({
      track: SETTINGS_SEGMENT_TRACK_RADIUS_CLASS,
      indicator: SETTINGS_SEGMENT_INDICATOR_RADIUS_CLASS,
    });
    expect(SETTINGS_SEGMENT_TRACK_RADIUS_CLASS).toBe("rounded-lg");
    expect(SETTINGS_SEGMENT_INDICATOR_RADIUS_CLASS).toBe("rounded-md");
  });
});

describe("INLINE_SEGMENT_INDICATOR_RADIUS_CLASS", () => {
  it("matches inline segment item rounding", () => {
    expect(INLINE_SEGMENT_INDICATOR_RADIUS_CLASS).toBe("rounded-md");
  });
});
