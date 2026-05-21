import gsap from "gsap";

import { prefersReducedMotion } from "@/lib/utils/tray-panel-height-animation";

export const USAGE_METER_FILL_DURATION_S = 0.5;
export const USAGE_METER_FILL_EASE = "power2.out";

export function usageMeterFillTranslateX(percent: number): string {
  const clamped = Math.max(0, Math.min(100, percent));
  return `${-(100 - clamped)}%`;
}

export function animateUsageMeterFill(
  indicator: HTMLElement,
  fromPercent: number,
  toPercent: number,
): gsap.core.Tween | void {
  const fromX = usageMeterFillTranslateX(fromPercent);
  const toX = usageMeterFillTranslateX(toPercent);

  if (prefersReducedMotion() || fromPercent === toPercent) {
    gsap.set(indicator, { x: toX });
    return;
  }

  return gsap.fromTo(
    indicator,
    { x: fromX },
    {
      x: toX,
      duration: USAGE_METER_FILL_DURATION_S,
      ease: USAGE_METER_FILL_EASE,
      overwrite: "auto",
    },
  );
}
