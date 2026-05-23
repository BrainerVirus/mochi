import gsap from "gsap";

export const TRAY_INDICATOR_DURATION_S = 0.35;
export const TRAY_INDICATOR_EASE = "power3.inOut";

export interface IndicatorMetrics {
  x: number;
  width: number;
}

export type ActiveIndicatorPlan = { mode: "snap" } | { mode: "tween"; start?: IndicatorMetrics };

export interface ActiveIndicatorOptions {
  animate: boolean;
  handoffStart?: IndicatorMetrics | null;
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
  return {
    x: toNumericGsapProperty(gsap.getProperty(indicator, "x")),
    width: toNumericGsapProperty(gsap.getProperty(indicator, "width")),
  };
}

export function isHoverIndicatorVisible(indicator: HTMLElement): boolean {
  return toNumericGsapProperty(gsap.getProperty(indicator, "autoAlpha")) > 0;
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

export function resolveHoverHandoffStart({
  hoveredId,
  targetTabId,
  hoverVisible,
  hoverMetrics,
}: {
  hoveredId: string | null;
  targetTabId: string;
  hoverVisible: boolean;
  hoverMetrics: IndicatorMetrics;
}): IndicatorMetrics | null {
  if (!hoverVisible || hoveredId !== targetTabId) {
    return null;
  }

  return hoverMetrics;
}

export function resolveActiveIndicatorPlan({
  target,
  current,
  handoffStart,
  animate,
  reducedMotion,
}: {
  target: IndicatorMetrics;
  current: IndicatorMetrics;
  handoffStart: IndicatorMetrics | null;
  animate: boolean;
  reducedMotion: boolean;
}): ActiveIndicatorPlan {
  if (!animate || reducedMotion) {
    return { mode: "snap" };
  }

  if (handoffStart) {
    return { mode: "tween", start: handoffStart };
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
    handoffStart: options.handoffStart ?? null,
    animate: options.animate,
    reducedMotion,
  });

  if (plan.mode === "snap") {
    gsap.set(indicator, { x: metrics.x, width: metrics.width, autoAlpha: 1, force3D: true });
    return;
  }

  if (plan.start) {
    gsap.set(indicator, {
      x: plan.start.x,
      width: plan.start.width,
      autoAlpha: 1,
      force3D: true,
    });
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
) {
  if (reducedMotion) {
    gsap.set(indicator, { ...metrics, opacity: hoveredId === activeValue ? 0 : 1, force3D: true });
    return;
  }

  if (quickTo) {
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
  gsap.killTweensOf(indicator, "autoAlpha");
  gsap.to(indicator, {
    autoAlpha: 0,
    duration: animate && !prefersReducedMotion() ? 0.15 : 0,
    overwrite: "auto",
  });
}

export function mergeHoverIntoActiveStart(
  hoverIndicator: HTMLElement,
  hoveredId: string,
  targetTabId: string,
): IndicatorMetrics | null {
  const handoff = resolveHoverHandoffStart({
    hoveredId,
    targetTabId,
    hoverVisible: isHoverIndicatorVisible(hoverIndicator),
    hoverMetrics: readIndicatorMetrics(hoverIndicator),
  });

  if (!handoff) {
    return null;
  }

  gsap.killTweensOf(hoverIndicator);
  hideHoverIndicator(hoverIndicator, false);
  return handoff;
}

export function shouldHideHoverOnLeave(suppressHoverEnd: boolean): boolean {
  return !suppressHoverEnd;
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

export function syncHoverSegmentIndicator(
  track: HTMLElement | null,
  indicator: HTMLElement | null,
  item: HTMLButtonElement | undefined,
  quickTo: HoverIndicatorQuickTo | null,
  activeValue: string,
  hoveredId: string,
) {
  if (!track || !indicator || !item) {
    return;
  }

  applyHoverIndicatorPosition(
    indicator,
    measureSegmentItem(track, item),
    quickTo,
    activeValue,
    hoveredId,
  );
}

export function observeSegmentTrackResize(
  track: HTMLElement,
  onResize: () => void,
): ResizeObserver {
  const resizeObserver = new ResizeObserver(onResize);
  resizeObserver.observe(track);
  return resizeObserver;
}
