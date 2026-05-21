import gsap from "gsap";

export const TRAY_INDICATOR_DURATION_S = 0.3;
export const TRAY_INDICATOR_EASE = "power3.out";

export interface IndicatorMetrics {
  x: number;
  width: number;
}

export function measureSegmentItem(_track: HTMLElement, item: HTMLElement): IndicatorMetrics {
  const group = item.parentElement;
  const groupLeft = group?.offsetLeft ?? 0;
  return {
    x: groupLeft + item.offsetLeft,
    width: item.offsetWidth,
  };
}

export function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function applyActiveIndicatorPosition(
  indicator: HTMLElement,
  metrics: IndicatorMetrics,
  options: { animate: boolean; instant: boolean },
) {
  const { x, width } = metrics;

  if (prefersReducedMotion() || options.instant || !options.animate) {
    gsap.set(indicator, { x, width, opacity: 1 });
    return;
  }

  gsap.to(indicator, {
    x,
    width,
    opacity: 1,
    duration: TRAY_INDICATOR_DURATION_S,
    ease: TRAY_INDICATOR_EASE,
    overwrite: "auto",
  });
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
    gsap.set(indicator, { ...metrics, opacity: hoveredId === activeValue ? 0 : 1 });
    return;
  }

  if (quickTo) {
    quickTo.x(metrics.x);
    quickTo.width(metrics.width);
  } else {
    gsap.set(indicator, metrics);
  }

  gsap.to(indicator, {
    opacity: hoveredId === activeValue ? 0 : 1,
    duration: 0.15,
    overwrite: "auto",
  });
}

export function hideHoverIndicator(indicator: HTMLElement, animate: boolean) {
  gsap.to(indicator, {
    opacity: 0,
    duration: animate && !prefersReducedMotion() ? 0.15 : 0,
    overwrite: "auto",
  });
}

export function syncActiveSegmentIndicator(
  track: HTMLElement | null,
  indicator: HTMLElement | null,
  item: HTMLButtonElement | undefined,
  instant: boolean,
) {
  if (!track || !indicator || !item) {
    return;
  }

  applyActiveIndicatorPosition(indicator, measureSegmentItem(track, item), {
    animate: !instant,
    instant,
  });
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
