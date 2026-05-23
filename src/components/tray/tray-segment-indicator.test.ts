import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  applyActiveIndicatorPosition,
  applyHoverIndicatorPosition,
  computeIndicatorTarget,
  mergeHoverIntoActiveStart,
  metricsFromClientRects,
  readIndicatorMetrics,
  resolveActiveIndicatorPlan,
  resolveHoverHandoffStart,
  shouldAnimateActiveIndicator,
  shouldHideHoverOnLeave,
  type HoverIndicatorQuickTo,
} from "@/components/tray/tray-segment-indicator";

const gsapMocks = vi.hoisted(() => ({
  getProperty: vi.fn<(target: unknown, prop: string) => string | number>(),
  set: vi.fn<(target: unknown, vars: object) => void>(),
  to: vi.fn<(target: unknown, vars: object) => unknown>(),
  fromTo: vi.fn<(target: unknown, from: object, to: object) => unknown>(),
  killTweensOf: vi.fn<(target: unknown, prop?: string) => void>(),
  quickTo: vi.fn<(target: unknown, prop: string, vars: object) => (value: number) => void>(
    () => vi.fn<(value: number) => void>(),
  ),
}));

function mockIndicator(): HTMLElement {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- gsap target test double without DOM
  return { tagName: "DIV" } as HTMLElement;
}

function mockQuickTo(): HoverIndicatorQuickTo["x"] {
  const fn = vi.fn<(value: number) => void>();
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- gsap QuickTo test double
  return Object.assign(fn, { tween: {} }) as unknown as HoverIndicatorQuickTo["x"];
}

vi.mock("gsap", () => ({
  default: gsapMocks,
}));

describe("metricsFromClientRects", () => {
  it("returns position relative to the track using layout boxes", () => {
    expect(metricsFromClientRects({ left: 10 }, { left: 90, width: 60 })).toEqual({
      x: 80,
      width: 60,
    });
  });
});

describe("computeIndicatorTarget", () => {
  it("matches metricsFromClientRects for tab layout boxes", () => {
    expect(computeIndicatorTarget({ left: 0 }, { left: 120, width: 72 })).toEqual({
      x: 120,
      width: 72,
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

describe("resolveActiveIndicatorPlan", () => {
  const target = { x: 160, width: 72 };

  it("snaps on first layout when the pill has not been placed yet", () => {
    expect(
      resolveActiveIndicatorPlan({
        target,
        current: { x: 0, width: 0 },
        handoffStart: null,
        animate: true,
        reducedMotion: false,
      }),
    ).toEqual({ mode: "snap" });
  });

  it("tweens from the current pill position when changing tabs", () => {
    expect(
      resolveActiveIndicatorPlan({
        target,
        current: { x: 40, width: 68 },
        handoffStart: null,
        animate: true,
        reducedMotion: false,
      }),
    ).toEqual({ mode: "tween" });
  });

  it("tweens from the hover handoff position when click merges mid-hover", () => {
    expect(
      resolveActiveIndicatorPlan({
        target,
        current: { x: 40, width: 68 },
        handoffStart: { x: 95, width: 70 },
        animate: true,
        reducedMotion: false,
      }),
    ).toEqual({ mode: "tween", start: { x: 95, width: 70 } });
  });
});

describe("resolveHoverHandoffStart", () => {
  it("returns null when the hovered tab does not match the selection", () => {
    expect(
      resolveHoverHandoffStart({
        hoveredId: "alpha",
        targetTabId: "beta",
        hoverVisible: true,
        hoverMetrics: { x: 12, width: 64 },
      }),
    ).toBeNull();
  });

  it("returns hover metrics when clicking the hovered tab mid-tween", () => {
    expect(
      resolveHoverHandoffStart({
        hoveredId: "alpha",
        targetTabId: "alpha",
        hoverVisible: true,
        hoverMetrics: { x: 55, width: 66 },
      }),
    ).toEqual({ x: 55, width: 66 });
  });
});

describe("shouldHideHoverOnLeave", () => {
  it("keeps the active pill untouched while pointer-down suppresses hover end", () => {
    expect(shouldHideHoverOnLeave(true)).toBe(false);
    expect(shouldHideHoverOnLeave(false)).toBe(true);
  });
});

describe("applyHoverIndicatorPosition", () => {
  const indicator = mockIndicator();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("follows pointerenter tab positions via quickTo", () => {
    const quickX = mockQuickTo();
    const quickWidth = mockQuickTo();

    applyHoverIndicatorPosition(
      indicator,
      { x: 88, width: 64 },
      { x: quickX, width: quickWidth },
      "alpha",
      "beta",
      false,
    );

    expect(quickX).toHaveBeenCalledWith(88);
    expect(quickWidth).toHaveBeenCalledWith(64);
  });
});

describe("applyActiveIndicatorPosition", () => {
  const indicator = mockIndicator();

  beforeEach(() => {
    vi.clearAllMocks();
    gsapMocks.getProperty.mockImplementation((_target, prop) => {
      if (prop === "x") return 40;
      if (prop === "width") return 68;
      if (prop === "autoAlpha") return 1;
      return 0;
    });
  });

  it("tweens from the current pill position when changing tabs", () => {
    applyActiveIndicatorPosition(indicator, { x: 160, width: 72 }, {
      animate: true,
      reducedMotion: false,
    });

    expect(gsapMocks.fromTo).not.toHaveBeenCalled();
    expect(gsapMocks.to).toHaveBeenCalledWith(
      indicator,
      expect.objectContaining({
        x: 160,
        width: 72,
        overwrite: "auto",
      }),
    );
  });

  it("snaps on first layout without tweening from the origin", () => {
    gsapMocks.getProperty.mockImplementation((_target, prop) => {
      if (prop === "width") return 0;
      if (prop === "x") return 0;
      return 0;
    });

    applyActiveIndicatorPosition(indicator, { x: 40, width: 68 }, {
      animate: true,
      reducedMotion: false,
    });

    expect(gsapMocks.to).not.toHaveBeenCalled();
    expect(gsapMocks.set).toHaveBeenCalledWith(
      indicator,
      expect.objectContaining({ x: 40, width: 68, autoAlpha: 1 }),
    );
  });

  it("starts the active tween from the hover handoff position, not x=0", () => {
    applyActiveIndicatorPosition(
      indicator,
      { x: 120, width: 72 },
      { animate: true, reducedMotion: false, handoffStart: { x: 95, width: 70 } },
    );

    expect(gsapMocks.set).toHaveBeenCalledWith(
      indicator,
      expect.objectContaining({ x: 95, width: 70, autoAlpha: 1 }),
    );
    expect(gsapMocks.to).toHaveBeenCalledWith(
      indicator,
      expect.objectContaining({ x: 120, width: 72 }),
    );
  });
});

describe("mergeHoverIntoActiveStart", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    gsapMocks.getProperty.mockImplementation((_target, prop) => {
      if (prop === "x") return 55;
      if (prop === "width") return 66;
      if (prop === "autoAlpha") return 1;
      return 0;
    });
  });

  it("returns null when the hovered tab does not match the selection", () => {
    expect(mergeHoverIntoActiveStart(mockIndicator(), "alpha", "beta")).toBeNull();
  });

  it("returns the hover pill metrics when clicking the hovered tab", () => {
    expect(mergeHoverIntoActiveStart(mockIndicator(), "alpha", "alpha")).toEqual({
      x: 55,
      width: 66,
    });
  });
});

describe("readIndicatorMetrics", () => {
  it("reads numeric gsap transform values from the indicator element", () => {
    gsapMocks.getProperty.mockImplementation((_target, prop) => {
      if (prop === "x") return "48px";
      if (prop === "width") return 72;
      return 0;
    });

    expect(readIndicatorMetrics(mockIndicator())).toEqual({ x: 48, width: 72 });
  });
});
