import gsap from "gsap";
import { useLayoutEffect, useRef, type RefObject } from "react";

import {
  animateUsageMeterFill,
  resolveUsageMeterFillStartPercent,
  usageMeterFillTranslateX,
} from "@/lib/utils/usage-meter-fill-animation";
import { prefersReducedMotion } from "@/lib/utils/tray-panel-height-animation";
import { runWhenTrayTabFillReady } from "@/lib/utils/tray-tab-fill-scheduler";

function resolveUsageMeterIndicator(
  meterRef: RefObject<HTMLDivElement | null>,
  indicatorRef: RefObject<HTMLDivElement | null>,
): HTMLElement | null {
  return (
    indicatorRef.current ??
    meterRef.current?.querySelector<HTMLElement>('[data-slot="progress-indicator"]') ??
    null
  );
}

export function useUsageMeterFill(
  meterRef: RefObject<HTMLDivElement | null>,
  indicatorRef: RefObject<HTMLDivElement | null>,
  clampedPercent: number,
  fillActivationKey: string,
): void {
  const previousPercentRef = useRef<number | null>(null);
  const previousActivationKeyRef = useRef<string | null>(null);

  useLayoutEffect(() => {
    const indicator = resolveUsageMeterIndicator(meterRef, indicatorRef);
    if (!indicator) {
      return undefined;
    }

    const from = resolveUsageMeterFillStartPercent(
      previousPercentRef.current,
      fillActivationKey,
      previousActivationKeyRef.current,
    );
    previousActivationKeyRef.current = fillActivationKey;
    previousPercentRef.current = clampedPercent;

    const ctx = gsap.context(() => {}, meterRef);

    const startFill = () => {
      if (prefersReducedMotion() || from === clampedPercent) {
        gsap.set(indicator, { x: usageMeterFillTranslateX(clampedPercent) });
        return;
      }

      gsap.set(indicator, { x: usageMeterFillTranslateX(from) });
      animateUsageMeterFill(indicator, from, clampedPercent);
    };

    const cancelReady = runWhenTrayTabFillReady(startFill);

    return () => {
      cancelReady();
      gsap.killTweensOf(indicator);
      ctx.revert();
    };
  }, [clampedPercent, fillActivationKey, indicatorRef, meterRef]);
}
