import gsap from "gsap";

export const INDICATOR_TRANSFORM_ORIGIN = "left center";

export const INDICATOR_COMPOSITOR_PROPS = {
  force3D: true,
  transformOrigin: INDICATOR_TRANSFORM_ORIGIN,
} as const;

function toNumericGsapProperty(value: string | number): number {
  if (typeof value === "number") {
    return value;
  }
  const parsed = Number.parseFloat(value);
  return Number.isNaN(parsed) ? 0 : parsed;
}

export function readIndicatorScaleX(indicator: HTMLElement): number {
  const scaleX = toNumericGsapProperty(gsap.getProperty(indicator, "scaleX"));
  return scaleX === 0 ? 1 : scaleX;
}

export function normalizeIndicatorLayout(
  indicator: HTMLElement,
  metrics: { x: number; width: number },
): { x: number; width: number } {
  gsap.set(indicator, {
    ...metrics,
    scaleX: 1,
    ...INDICATOR_COMPOSITOR_PROPS,
  });
  return metrics;
}

export function resolveIndicatorScaleX(layoutWidth: number, targetVisualWidth: number): number {
  if (layoutWidth <= 0) {
    return 1;
  }
  return targetVisualWidth / layoutWidth;
}
