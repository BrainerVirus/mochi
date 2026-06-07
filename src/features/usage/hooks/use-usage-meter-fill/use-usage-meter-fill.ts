import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { useRef, type RefObject } from "react";

import { prefersReducedMotion } from "@/lib/utils/tray-panel-height-animation";
import {
  animateUsageMeterFill,
  resolveUsageMeterFillStartPercent,
  usageMeterFillTranslateX,
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
    (_context, contextSafe) => {
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

      const startFill = contextSafe
        ? contextSafe(() => {
            runUsageMeterFill(indicator, from, clampedPercent);
          })
        : () => {
            runUsageMeterFill(indicator, from, clampedPercent);
          };

      startFill();

      return () => {
        gsap.killTweensOf(indicator);
      };
    },
    { dependencies: [clampedPercent, fillActivationKey], scope: meterRef, revertOnUpdate: true },
  );
}

function runUsageMeterFill(indicator: HTMLElement, from: number, to: number): void {
  if (prefersReducedMotion() || from === to) {
    gsap.set(indicator, { x: usageMeterFillTranslateX(to) });
    return;
  }

  gsap.set(indicator, { x: usageMeterFillTranslateX(from) });
  animateUsageMeterFill(indicator, from, to);
}
