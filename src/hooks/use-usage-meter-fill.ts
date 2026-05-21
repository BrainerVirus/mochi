import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { useRef, type RefObject } from "react";

import {
  resolveUsageMeterFillStartPercent,
  scheduleUsageMeterFill,
} from "@/lib/utils/usage-meter-fill-animation";

gsap.registerPlugin(useGSAP);

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

  useGSAP(
    () => {
      const indicator = resolveUsageMeterIndicator(meterRef, indicatorRef);
      if (!indicator) {
        return () => {};
      }

      const from = resolveUsageMeterFillStartPercent(
        previousPercentRef.current,
        fillActivationKey,
        previousActivationKeyRef.current,
      );
      previousActivationKeyRef.current = fillActivationKey;

      const cancelScheduled = scheduleUsageMeterFill(indicator, from, clampedPercent);
      previousPercentRef.current = clampedPercent;

      return () => {
        cancelScheduled();
      };
    },
    { dependencies: [clampedPercent, fillActivationKey], scope: meterRef, revertOnUpdate: true },
  );
}
