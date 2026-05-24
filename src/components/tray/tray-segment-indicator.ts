import gsap from "gsap";

import type { TraySegmentIndicatorCommand } from "@/components/tray/tray-segment-indicator-machine";
import {
  INDICATOR_COMPOSITOR_PROPS,
  normalizeIndicatorLayout,
  readIndicatorScaleX,
  resolveIndicatorScaleX,
} from "@/components/tray/tray-segment-indicator-transform";

export { observeSegmentTrackResize } from "@/components/tray/segment-track-resize-observer";
export type { SegmentTrackResizeObserver } from "@/components/tray/segment-track-resize-observer";
export { resolveIndicatorScaleX } from "@/components/tray/tray-segment-indicator-transform";

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

export function computeIndicatorTarget(
  trackRect: Pick<DOMRect, "left">,
  itemRect: Pick<DOMRect, "left" | "width">,
): IndicatorMetrics {
  return metricsFromClientRects(trackRect, itemRect);
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
  const layoutWidth = toNumericGsapProperty(gsap.getProperty(indicator, "width"));
  return {
    x: toNumericGsapProperty(gsap.getProperty(indicator, "x")),
    width: layoutWidth * readIndicatorScaleX(indicator),
  };
}

export function isIndicatorPlaced(metrics: IndicatorMetrics): boolean {
  return metrics.width > 0;
}

export function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
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
  const reducedMotion = options.reducedMotion ?? prefersReducedMotion();
  const plan = resolveActiveIndicatorPlan({
    target: metrics,
    current: readIndicatorMetrics(indicator),
    animate: options.animate,
    reducedMotion,
  });

  if (plan.mode === "snap") {
    gsap.set(indicator, {
      x: metrics.x,
      width: metrics.width,
      scaleX: 1,
      autoAlpha: 1,
      ...INDICATOR_COMPOSITOR_PROPS,
    });
    return;
  }

  const current = readIndicatorMetrics(indicator);
  const startWidth = Math.max(current.width, 1);

  gsap.set(indicator, {
    width: startWidth,
    scaleX: 1,
    x: current.x,
    ...INDICATOR_COMPOSITOR_PROPS,
  });

  gsap.to(indicator, {
    x: metrics.x,
    scaleX: resolveIndicatorScaleX(startWidth, metrics.width),
    autoAlpha: 1,
    duration: TRAY_INDICATOR_DURATION_S,
    ease: TRAY_INDICATOR_EASE,
    overwrite: "auto",
    onComplete: () => {
      normalizeIndicatorLayout(indicator, metrics);
    },
  });
}

export interface HoverIndicatorQuickTo {
  x: gsap.QuickToFunc;
  scaleX: gsap.QuickToFunc;
}

export function createHoverIndicatorQuickTo(indicator: HTMLElement): HoverIndicatorQuickTo {
  return {
    x: gsap.quickTo(indicator, "x", {
      duration: TRAY_INDICATOR_DURATION_S,
      ease: TRAY_INDICATOR_EASE,
    }),
    scaleX: gsap.quickTo(indicator, "scaleX", {
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
  if (reducedMotion) {
    gsap.set(indicator, {
      x: metrics.x,
      width: metrics.width,
      scaleX: 1,
      opacity: hoveredId === activeValue ? 0 : 1,
      ...INDICATOR_COMPOSITOR_PROPS,
    });
    return;
  }

  if (fresh) {
    gsap.set(indicator, {
      x: metrics.x,
      width: metrics.width,
      scaleX: 1,
      ...INDICATOR_COMPOSITOR_PROPS,
    });
  } else if (quickTo) {
    const layoutWidth = toNumericGsapProperty(gsap.getProperty(indicator, "width"));
    quickTo.x(metrics.x);
    quickTo.scaleX(resolveIndicatorScaleX(layoutWidth, metrics.width));
  } else {
    gsap.set(indicator, {
      x: metrics.x,
      width: metrics.width,
      scaleX: 1,
      ...INDICATOR_COMPOSITOR_PROPS,
    });
  }

  gsap.to(indicator, {
    autoAlpha: hoveredId === activeValue ? 0 : 1,
    duration: 0.15,
    overwrite: "auto",
  });
}

export function hideHoverIndicator(indicator: HTMLElement, animate: boolean) {
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
  if (!track || !indicator || !item) {
    return null;
  }

  const metrics = measureSegmentItem(track, item);
  applyActiveIndicatorPosition(indicator, metrics, options);
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
    if (!context.track || !context.hoverIndicator || !item) {
      return;
    }

    applyHoverIndicatorPosition(
      context.hoverIndicator,
      measureSegmentItem(context.track, item),
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
