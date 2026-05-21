import gsap from "gsap";

import { prefersReducedMotion } from "@/lib/utils/tray-panel-height-animation";

export const USAGE_METER_FILL_DURATION_S = 0.5;
export const USAGE_METER_FILL_EASE = "power2.out";

export function usageMeterFillTranslateX(percent: number): string {
  const clamped = Math.max(0, Math.min(100, percent));
  return `${-(100 - clamped)}%`;
}

/** Restarts from empty on tab switch; keeps prior value for usage refresh on the same tab. */
export function resolveUsageMeterFillStartPercent(
  previousPercent: number | null,
  activationKey: string,
  previousActivationKey: string | null,
): number {
  if (previousActivationKey !== activationKey) {
    return 0;
  }

  return previousPercent ?? 0;
}

export function scheduleUsageMeterFill(
  indicator: HTMLElement,
  fromPercent: number,
  toPercent: number,
): () => void {
  if (prefersReducedMotion() || fromPercent === toPercent) {
    gsap.set(indicator, { x: usageMeterFillTranslateX(toPercent) });
    return () => {};
  }

  gsap.set(indicator, { x: usageMeterFillTranslateX(fromPercent) });

  let innerFrame = 0;
  const outerFrame = requestAnimationFrame(() => {
    innerFrame = requestAnimationFrame(() => {
      animateUsageMeterFill(indicator, fromPercent, toPercent);
    });
  });

  return () => {
    cancelAnimationFrame(outerFrame);
    if (innerFrame) {
      cancelAnimationFrame(innerFrame);
    }
    gsap.killTweensOf(indicator);
  };
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
