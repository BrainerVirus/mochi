import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  applyActiveIndicatorPosition,
  applyHoverIndicatorPosition,
  computeIndicatorTarget,
  metricsFromClientRects,
  readIndicatorMetrics,
  releaseSegmentIndicators,
  resolveActiveIndicatorPlan,
  shouldAnimateActiveIndicator,
  type HoverIndicatorQuickTo,
} from "@/components/tray/tray-segment-indicator";

const gsapMocks = vi.hoisted(() => ({
  getProperty: vi.fn<(target: unknown, prop: string) => string | number>(),
  set: vi.fn<(target: unknown, vars: object) => void>(),
  to: vi.fn<(target: unknown, vars: object) => unknown>(),
  fromTo: vi.fn<(target: unknown, from: object, to: object) => unknown>(),
  killTweensOf: vi.fn<(target: unknown, prop?: string) => void>(),
  quickTo: vi.fn<(target: unknown, prop: string, vars: object) => (value: number) => void>(() =>
    vi.fn<(value: number) => void>(),
  ),
}));

function mockIndicator(): HTMLElement {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- gsap target test double without DOM
  return { tagName: "DIV", isConnected: true } as HTMLElement;
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
        animate: true,
        reducedMotion: false,
      }),
    ).toEqual({ mode: "tween" });
  });

  it("keeps active movement anchored to the current active pill during hover clicks", () => {
    expect(
      resolveActiveIndicatorPlan({
        target,
        current: { x: 40, width: 68 },
        animate: true,
        reducedMotion: false,
      }),
    ).toEqual({ mode: "tween" });
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

  it("places fresh hover targets without gliding from the hidden previous tab", () => {
    const quickX = mockQuickTo();
    const quickWidth = mockQuickTo();

    applyHoverIndicatorPosition(
      indicator,
      { x: 144, width: 72 },
      { x: quickX, width: quickWidth },
      "alpha",
      "delta",
      false,
      true,
    );

    expect(quickX).not.toHaveBeenCalled();
    expect(quickWidth).not.toHaveBeenCalled();
    expect(gsapMocks.set).toHaveBeenCalledWith(
      indicator,
      expect.objectContaining({ x: 144, width: 72 }),
    );
    expect(gsapMocks.to).toHaveBeenCalledWith(indicator, expect.objectContaining({ autoAlpha: 1 }));
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
    applyActiveIndicatorPosition(
      indicator,
      { x: 160, width: 72 },
      {
        animate: true,
        reducedMotion: false,
      },
    );

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

    applyActiveIndicatorPosition(
      indicator,
      { x: 40, width: 68 },
      {
        animate: true,
        reducedMotion: false,
      },
    );

    expect(gsapMocks.to).not.toHaveBeenCalled();
    expect(gsapMocks.set).toHaveBeenCalledWith(
      indicator,
      expect.objectContaining({ x: 40, width: 68, autoAlpha: 1 }),
    );
  });

  it("does not teleport active to hover metrics before tweening", () => {
    applyActiveIndicatorPosition(
      indicator,
      { x: 120, width: 72 },
      { animate: true, reducedMotion: false },
    );

    expect(gsapMocks.set).not.toHaveBeenCalled();
    expect(gsapMocks.killTweensOf).not.toHaveBeenCalled();
    expect(gsapMocks.to).toHaveBeenCalledWith(
      indicator,
      expect.objectContaining({ x: 120, width: 72 }),
    );
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

describe("releaseSegmentIndicators", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("kills GSAP tweens on both indicator layers", () => {
    const active = mockIndicator();
    const hover = mockIndicator();

    releaseSegmentIndicators(active, hover);

    expect(gsapMocks.killTweensOf).toHaveBeenCalledWith(active);
    expect(gsapMocks.killTweensOf).toHaveBeenCalledWith(hover);
  });
});
