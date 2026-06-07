import gsap from "gsap";

import type { TraySegmentIndicatorCommand } from "@/features/tray/components/tray-segment-indicator-machine";

export { observeSegmentTrackResize } from "@/features/tray/components/segment-track-resize-observer";
export type { SegmentTrackResizeObserver } from "@/features/tray/components/segment-track-resize-observer";

export const TRAY_INDICATOR_DURATION_S = 0.35;
export const TRAY_INDICATOR_EASE = "power3.inOut";

export interface IndicatorMetrics {
  x: number;
  width: number;
}

export type ActiveIndicatorPlan = { mode: "snap" } | { mode: "tween" };

export interface ActiveIndicatorOptions {
  animate: boolean;
  reducedMotion?: boolean;
}

export function metricsFromClientRects(
  trackRect: Pick<DOMRect, "left">,
  itemRect: Pick<DOMRect, "left" | "width">,
): IndicatorMetrics {
  return {
    x: itemRect.left - trackRect.left,
    width: itemRect.width,
  };
}

export function measureSegmentItem(track: HTMLElement, item: HTMLElement): IndicatorMetrics {
  return metricsFromClientRects(track.getBoundingClientRect(), item.getBoundingClientRect());
}

function toNumericGsapProperty(value: string | number): number {
  if (typeof value === "number") {
    return value;
  }
  const parsed = Number.parseFloat(value);
  return Number.isNaN(parsed) ? 0 : parsed;
}

export function readIndicatorMetrics(indicator: HTMLElement): IndicatorMetrics {
  return {
    x: toNumericGsapProperty(gsap.getProperty(indicator, "x")),
    width: toNumericGsapProperty(gsap.getProperty(indicator, "width")),
  };
}

export function isIndicatorPlaced(metrics: IndicatorMetrics): boolean {
  return metrics.width > 0;
}

export function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function isLiveIndicator(indicator: HTMLElement | null): indicator is HTMLElement {
  return indicator !== null && indicator.isConnected;
}

function liveSegmentTarget(
  track: HTMLElement | null,
  indicator: HTMLElement | null,
  item: HTMLElement | undefined,
): { track: HTMLElement; indicator: HTMLElement; item: HTMLElement } | null {
  if (
    track === null ||
    !track.isConnected ||
    !isLiveIndicator(indicator) ||
    item === undefined ||
    !item.isConnected
  ) {
    return null;
  }

  return { track, indicator, item };
}

export function releaseSegmentIndicators(
  activeIndicator: HTMLElement | null,
  hoverIndicator: HTMLElement | null,
): void {
  if (activeIndicator) {
    gsap.killTweensOf(activeIndicator);
  }
  if (hoverIndicator) {
    gsap.killTweensOf(hoverIndicator);
  }
}

export function shouldAnimateActiveIndicator(
  prevMetrics: IndicatorMetrics | null,
  animate: boolean,
  reducedMotion = prefersReducedMotion(),
): boolean {
  return !reducedMotion && animate && prevMetrics !== null;
}

export function resolveActiveIndicatorPlan({
  target,
  current,
  animate,
  reducedMotion,
}: {
  target: IndicatorMetrics;
  current: IndicatorMetrics;
  animate: boolean;
  reducedMotion: boolean;
}): ActiveIndicatorPlan {
  if (!animate || reducedMotion) {
    return { mode: "snap" };
  }

  if (!isIndicatorPlaced(current)) {
    return { mode: "snap" };
  }

  if (current.x === target.x && current.width === target.width) {
    return { mode: "snap" };
  }

  return { mode: "tween" };
}

export function applyActiveIndicatorPosition(
  indicator: HTMLElement,
  metrics: IndicatorMetrics,
  options: ActiveIndicatorOptions,
) {
  if (!isLiveIndicator(indicator)) {
    return;
  }

  const reducedMotion = options.reducedMotion ?? prefersReducedMotion();
  const plan = resolveActiveIndicatorPlan({
    target: metrics,
    current: readIndicatorMetrics(indicator),
    animate: options.animate,
    reducedMotion,
  });

  if (plan.mode === "snap") {
    gsap.set(indicator, { x: metrics.x, width: metrics.width, autoAlpha: 1, force3D: true });
    return;
  }

  gsap.to(indicator, {
    x: metrics.x,
    width: metrics.width,
    autoAlpha: 1,
    duration: TRAY_INDICATOR_DURATION_S,
    ease: TRAY_INDICATOR_EASE,
    overwrite: "auto",
  });
}

export interface HoverIndicatorQuickTo {
  x: gsap.QuickToFunc;
  width: gsap.QuickToFunc;
}

export function createHoverIndicatorQuickTo(indicator: HTMLElement): HoverIndicatorQuickTo {
  return {
    x: gsap.quickTo(indicator, "x", {
      duration: TRAY_INDICATOR_DURATION_S,
      ease: TRAY_INDICATOR_EASE,
    }),
    width: gsap.quickTo(indicator, "width", {
      duration: TRAY_INDICATOR_DURATION_S,
      ease: TRAY_INDICATOR_EASE,
    }),
  };
}

export function applyHoverIndicatorPosition(
  indicator: HTMLElement,
  metrics: IndicatorMetrics,
  quickTo: HoverIndicatorQuickTo | null,
  activeValue: string,
  hoveredId: string,
  reducedMotion = prefersReducedMotion(),
  fresh = false,
) {
  if (!isLiveIndicator(indicator)) {
    return;
  }

  if (reducedMotion) {
    gsap.set(indicator, { ...metrics, opacity: hoveredId === activeValue ? 0 : 1, force3D: true });
    return;
  }

  if (fresh) {
    gsap.set(indicator, { ...metrics, force3D: true });
  } else if (quickTo) {
    quickTo.x(metrics.x);
    quickTo.width(metrics.width);
  } else {
    gsap.set(indicator, { ...metrics, force3D: true });
  }

  gsap.to(indicator, {
    autoAlpha: hoveredId === activeValue ? 0 : 1,
    duration: 0.15,
    overwrite: "auto",
  });
}

export function hideHoverIndicator(indicator: HTMLElement, animate: boolean) {
  if (!isLiveIndicator(indicator)) {
    return;
  }

  gsap.killTweensOf(indicator, "autoAlpha");
  gsap.to(indicator, {
    autoAlpha: 0,
    duration: animate && !prefersReducedMotion() ? 0.15 : 0,
    overwrite: "auto",
  });
}

export function syncActiveSegmentIndicator(
  track: HTMLElement | null,
  indicator: HTMLElement | null,
  item: HTMLButtonElement | undefined,
  options: ActiveIndicatorOptions,
) {
  const target = liveSegmentTarget(track, indicator, item);
  if (!target) {
    return null;
  }

  const metrics = measureSegmentItem(target.track, target.item);
  applyActiveIndicatorPosition(target.indicator, metrics, options);
  return metrics;
}

export interface TraySegmentIndicatorExecutorContext {
  track: HTMLElement | null;
  hoverIndicator: HTMLElement | null;
  activeIndicator: HTMLElement | null;
  itemRefs: Map<string, HTMLButtonElement>;
  hoverQuickTo: HoverIndicatorQuickTo | null;
  activeValue: string;
  reducedMotion?: boolean;
  resetHoverQuickTo?: () => void;
}

function itemForCommand(
  itemRefs: Map<string, HTMLButtonElement>,
  tabId: string,
): HTMLButtonElement | undefined {
  return itemRefs.get(tabId);
}

export function executeTraySegmentIndicatorCommand(
  command: TraySegmentIndicatorCommand,
  context: TraySegmentIndicatorExecutorContext,
) {
  if (command.type === "hideHover") {
    if (context.hoverIndicator) {
      hideHoverIndicator(context.hoverIndicator, !command.immediate);
    }
    return;
  }

  if (command.type === "placeHover" || command.type === "moveHover") {
    const item = itemForCommand(context.itemRefs, command.tabId);
    const target = liveSegmentTarget(context.track, context.hoverIndicator, item);
    if (!target) {
      return;
    }

    applyHoverIndicatorPosition(
      target.indicator,
      measureSegmentItem(target.track, target.item),
      command.type === "moveHover" ? context.hoverQuickTo : null,
      context.activeValue,
      command.tabId,
      context.reducedMotion,
      command.type === "placeHover",
    );
    if (command.type === "placeHover") {
      context.resetHoverQuickTo?.();
    }
    return;
  }

  const item = itemForCommand(context.itemRefs, command.tabId);
  syncActiveSegmentIndicator(context.track, context.activeIndicator, item, {
    animate: command.type === "moveActive",
    reducedMotion: context.reducedMotion,
  });
}
