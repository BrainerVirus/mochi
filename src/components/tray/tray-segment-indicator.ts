import gsap from "gsap";

export const TRAY_INDICATOR_DURATION_S = 0.35;
export const TRAY_INDICATOR_EASE = "power3.inOut";

export interface IndicatorMetrics {
  x: number;
  width: number;
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

export function applyActiveIndicatorPosition(
  indicator: HTMLElement,
  metrics: IndicatorMetrics,
  prevMetrics: IndicatorMetrics | null,
  animate: boolean,
) {
  const { x, width } = metrics;

  if (!shouldAnimateActiveIndicator(prevMetrics, animate)) {
    gsap.set(indicator, { x, width, autoAlpha: 1, force3D: true });
    return;
  }

  gsap.fromTo(
    indicator,
    { x: prevMetrics!.x, width: prevMetrics!.width, autoAlpha: 1 },
    {
      x,
      width,
      autoAlpha: 1,
      duration: TRAY_INDICATOR_DURATION_S,
      ease: TRAY_INDICATOR_EASE,
      overwrite: "auto",
    },
  );
}

export function createHoverIndicatorQuickTo(indicator: HTMLElement) {
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
  quickTo: { x: gsap.QuickToFunc; width: gsap.QuickToFunc } | null,
  activeValue: string,
  hoveredId: string,
) {
  if (prefersReducedMotion()) {
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
  prevMetrics: IndicatorMetrics | null,
  animate: boolean,
) {
  if (!track || !indicator || !item) {
    return null;
  }

  const metrics = measureSegmentItem(track, item);
  applyActiveIndicatorPosition(indicator, metrics, prevMetrics, animate);
  return metrics;
}

export function syncHoverSegmentIndicator(
  track: HTMLElement | null,
  indicator: HTMLElement | null,
  item: HTMLButtonElement | undefined,
  quickTo: { x: gsap.QuickToFunc; width: gsap.QuickToFunc } | null,
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
