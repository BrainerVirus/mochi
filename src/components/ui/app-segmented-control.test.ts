import { describe, expect, it } from "vitest";

import {
  PAGE_TAB_SEGMENT_DEFAULTS,
  TRAY_SEGMENT_ROW_HEIGHT,
} from "@/components/tray/tray-segmented-control";
import {
  APP_SEGMENT_INDICATOR_RADIUS_CLASS,
  APP_SEGMENT_TRACK_RADIUS_CLASS,
  usesPageTabIndicators,
} from "@/components/ui/app-segmented-control";

describe("usesPageTabIndicators", () => {
  it("enables GSAP hover/active pills for page tabs", () => {
    expect(usesPageTabIndicators("page-tabs")).toBe(true);
  });

  it("disables GSAP hover/active pills for inline controls", () => {
    expect(usesPageTabIndicators("inline")).toBe(false);
  });
});

describe("PAGE_TAB_SEGMENT_DEFAULTS", () => {
  it("matches tray tab strip layout", () => {
    expect(PAGE_TAB_SEGMENT_DEFAULTS).toEqual({
      variant: "page-tabs",
      rowHeight: TRAY_SEGMENT_ROW_HEIGHT,
      stretchItems: false,
    });
  });
});

describe("page-tab segment radius", () => {
  it("uses fixed radius classes independent of .app-window --radius", () => {
    expect(APP_SEGMENT_INDICATOR_RADIUS_CLASS).toBe("rounded-app-segment-indicator");
    expect(APP_SEGMENT_TRACK_RADIUS_CLASS).toBe("rounded-app-segment-track");
  });
});
