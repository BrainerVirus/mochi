import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  applyActiveIndicatorPosition,
  metricsFromClientRects,
  TRAY_INDICATOR_DURATION_S,
  TRAY_INDICATOR_EASE,
} from "@/components/tray/tray-segment-indicator";

const gsapMocks = vi.hoisted(() => ({
  fromTo: vi.fn(),
  set: vi.fn(),
  to: vi.fn(),
  quickTo: vi.fn(() => vi.fn()),
}));

vi.mock("gsap", () => ({
  default: gsapMocks,
}));

describe("metricsFromClientRects", () => {
  it("returns position relative to the track using layout boxes", () => {
    expect(
      metricsFromClientRects({ left: 10 }, { left: 90, width: 60 }),
    ).toEqual({ x: 80, width: 60 });
  });
});

describe("applyActiveIndicatorPosition", () => {
  const indicator = document.createElement("div");
  const next = { x: 120, width: 72 };
  const prev = { x: 40, width: 68 };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal(
      "matchMedia",
      vi.fn().mockReturnValue({ matches: false, addEventListener: vi.fn() }),
    );
  });

  it("snaps on first layout without a previous position", () => {
    applyActiveIndicatorPosition(indicator, next, null, true);

    expect(gsapMocks.set).toHaveBeenCalledWith(indicator, {
      x: next.x,
      width: next.width,
      opacity: 1,
      force3D: true,
    });
    expect(gsapMocks.fromTo).not.toHaveBeenCalled();
  });

  it("morphs from the previous position when animating segment changes", () => {
    applyActiveIndicatorPosition(indicator, next, prev, true);

    expect(gsapMocks.fromTo).toHaveBeenCalledWith(
      indicator,
      { x: prev.x, width: prev.width, opacity: 1 },
      {
        x: next.x,
        width: next.width,
        opacity: 1,
        duration: TRAY_INDICATOR_DURATION_S,
        ease: TRAY_INDICATOR_EASE,
        overwrite: "auto",
      },
    );
  });

  it("snaps when animation is disabled", () => {
    applyActiveIndicatorPosition(indicator, next, prev, false);

    expect(gsapMocks.set).toHaveBeenCalledWith(indicator, {
      x: next.x,
      width: next.width,
      opacity: 1,
      force3D: true,
    });
    expect(gsapMocks.fromTo).not.toHaveBeenCalled();
  });
});
