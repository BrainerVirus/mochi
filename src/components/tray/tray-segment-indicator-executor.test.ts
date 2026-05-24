import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  executeTraySegmentIndicatorCommand,
  type HoverIndicatorQuickTo,
} from "@/components/tray/tray-segment-indicator";

const gsapMocks = vi.hoisted(() => ({
  getProperty: vi.fn<(target: unknown, prop: string) => string | number>(),
  set: vi.fn<(target: unknown, vars: object) => void>(),
  to: vi.fn<(target: unknown, vars: object) => unknown>(),
  killTweensOf: vi.fn<(target: unknown, prop?: string) => void>(),
  quickTo: vi.fn<(target: unknown, prop: string, vars: object) => (value: number) => void>(() =>
    vi.fn<(value: number) => void>(),
  ),
}));

vi.mock("gsap", () => ({
  default: gsapMocks,
}));

function mockIndicator(): HTMLElement {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- gsap target test double
  return { tagName: "DIV", isConnected: true } as HTMLElement;
}

function mockMeasuredButton(left: number, width: number): HTMLButtonElement {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- focused DOM geometry test double
  return {
    getBoundingClientRect: () => ({ left, width }),
  } as HTMLButtonElement;
}

function mockMeasuredTrack(left: number, width: number): HTMLElement {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- focused DOM geometry test double
  return {
    getBoundingClientRect: () => ({ left, width }),
  } as HTMLElement;
}

function mockQuickTo(): HoverIndicatorQuickTo["x"] {
  const fn = vi.fn<(value: number) => void>();
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- gsap QuickTo test double
  return Object.assign(fn, { tween: {} }) as unknown as HoverIndicatorQuickTo["x"];
}

const track = mockMeasuredTrack(10, 260);
const hoverIndicator = mockIndicator();
const activeIndicator = mockIndicator();
const codex = mockMeasuredButton(82, 64);
const cursor = mockMeasuredButton(154, 72);

function resetGsapMocks() {
  vi.clearAllMocks();
  gsapMocks.getProperty.mockImplementation((_target, prop) => {
    if (prop === "x") return 24;
    if (prop === "width") return 64;
    if (prop === "autoAlpha") return 1;
    return 0;
  });
}

describe("executeTraySegmentIndicatorCommand hover commands", () => {
  beforeEach(() => {
    resetGsapMocks();
  });

  it("places hover by setting x and width before fading it in", () => {
    const quickX = mockQuickTo();
    const quickWidth = mockQuickTo();
    const resetHoverQuickTo = vi.fn<() => void>();

    executeTraySegmentIndicatorCommand(
      { type: "placeHover", tabId: "codex" },
      {
        track,
        hoverIndicator,
        activeIndicator,
        itemRefs: new Map([["codex", codex]]),
        hoverQuickTo: { x: quickX, width: quickWidth },
        activeValue: "overview",
        reducedMotion: false,
        resetHoverQuickTo,
      },
    );

    expect(quickX).not.toHaveBeenCalled();
    expect(quickWidth).not.toHaveBeenCalled();
    expect(gsapMocks.set).toHaveBeenCalledWith(
      hoverIndicator,
      expect.objectContaining({ x: 72, width: 64 }),
    );
    expect(gsapMocks.to).toHaveBeenCalledWith(
      hoverIndicator,
      expect.objectContaining({ autoAlpha: 1 }),
    );
    expect(resetHoverQuickTo).toHaveBeenCalledOnce();
  });

  it("moves hover through quickTo when crossing between tabs", () => {
    const quickX = mockQuickTo();
    const quickWidth = mockQuickTo();

    executeTraySegmentIndicatorCommand(
      { type: "moveHover", tabId: "cursor" },
      {
        track,
        hoverIndicator,
        activeIndicator,
        itemRefs: new Map([["cursor", cursor]]),
        hoverQuickTo: { x: quickX, width: quickWidth },
        activeValue: "overview",
        reducedMotion: false,
      },
    );

    expect(quickX).toHaveBeenCalledWith(144);
    expect(quickWidth).toHaveBeenCalledWith(72);
    expect(gsapMocks.set).not.toHaveBeenCalled();
  });
});

describe("executeTraySegmentIndicatorCommand hide commands", () => {
  beforeEach(() => {
    resetGsapMocks();
  });

  it("hides hover without moving x or width", () => {
    const quickX = mockQuickTo();
    const quickWidth = mockQuickTo();

    executeTraySegmentIndicatorCommand(
      { type: "hideHover", immediate: true },
      {
        track,
        hoverIndicator,
        activeIndicator,
        itemRefs: new Map(),
        hoverQuickTo: { x: quickX, width: quickWidth },
        activeValue: "overview",
        reducedMotion: false,
      },
    );

    expect(quickX).not.toHaveBeenCalled();
    expect(quickWidth).not.toHaveBeenCalled();
    expect(gsapMocks.killTweensOf).toHaveBeenCalledWith(hoverIndicator, "autoAlpha");
    expect(gsapMocks.to).toHaveBeenCalledWith(
      hoverIndicator,
      expect.objectContaining({ autoAlpha: 0, duration: 0 }),
    );
  });
});

describe("executeTraySegmentIndicatorCommand active commands", () => {
  beforeEach(() => {
    resetGsapMocks();
  });

  it("moves active without setting hover metrics as its start", () => {
    executeTraySegmentIndicatorCommand(
      { type: "moveActive", tabId: "cursor" },
      {
        track,
        hoverIndicator,
        activeIndicator,
        itemRefs: new Map([["cursor", cursor]]),
        hoverQuickTo: null,
        activeValue: "overview",
        reducedMotion: false,
      },
    );

    expect(gsapMocks.set).not.toHaveBeenCalled();
    expect(gsapMocks.to).toHaveBeenCalledWith(
      activeIndicator,
      expect.objectContaining({ x: 144, width: 72 }),
    );
  });
});
