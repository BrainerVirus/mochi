import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { TextPlugin } from "gsap/TextPlugin";
import { useRef, type RefObject } from "react";

import { prefersReducedMotion } from "@/lib/utils/tray-panel-height-animation";
import {
  formatUsageMeterLeftLabel,
  resolveUsageMeterFillStartPercent,
} from "@/lib/utils/usage-meter-fill-animation";

gsap.registerPlugin(useGSAP, TextPlugin);

export function useUsageMeterLeftLabel(
  labelRef: RefObject<HTMLSpanElement | null>,
  percent: number,
  fillActivationKey: string,
): void {
  const previousPercentRef = useRef<number | null>(null);
  const previousActivationKeyRef = useRef<string | null>(null);

  useGSAP(
    () => {
      const label = labelRef.current;
      if (!label) {
        return undefined;
      }

      const from = resolveUsageMeterFillStartPercent(
        previousPercentRef.current,
        fillActivationKey,
        previousActivationKeyRef.current,
      );
      previousActivationKeyRef.current = fillActivationKey;
      previousPercentRef.current = percent;

      if (prefersReducedMotion() || from === percent) {
        gsap.set(label, { text: { value: formatUsageMeterLeftLabel(percent) } });
        return undefined;
      }

      const value = { percent: from };
      const tween = gsap.to(value, {
        percent,
        duration: 0.5,
        ease: "power2.out",
        overwrite: "auto",
        onStart: () => {
          gsap.set(label, { text: { value: formatUsageMeterLeftLabel(from) } });
        },
        onUpdate: () => {
          gsap.set(label, { text: { value: formatUsageMeterLeftLabel(value.percent) } });
        },
        onComplete: () => {
          gsap.set(label, { text: { value: formatUsageMeterLeftLabel(percent) } });
        },
      });

      return () => {
        tween.kill();
      };
    },
    { dependencies: [percent, fillActivationKey], scope: labelRef, revertOnUpdate: true },
  );
}
