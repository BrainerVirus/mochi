"use client";

import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { useRef, type RefObject } from "react";

gsap.registerPlugin(useGSAP);

export const SCROLL_OVERFLOW_FADE_DURATION_S = 0.2;
export const SCROLL_OVERFLOW_FADE_EASE = "power2.out";
export const SCROLL_OVERFLOW_SLIDE_PX = 4;

type SlideAxis = "x" | "y";

interface UseGsapOverflowVisibilityOptions {
  visible: boolean;
  /** Slide direction when hidden; omit for opacity-only fades. */
  slide?: "start" | "end";
  axis?: SlideAxis;
}

function hiddenOffset(side: "start" | "end", axis: SlideAxis): number {
  const magnitude = SCROLL_OVERFLOW_SLIDE_PX;
  if (axis === "x") {
    return side === "start" ? -magnitude : magnitude;
  }
  return side === "start" ? -magnitude : magnitude;
}

export function animateOverflowVisibility(
  element: HTMLElement,
  { visible, slide, axis = "x" }: UseGsapOverflowVisibilityOptions,
) {
  const hiddenOffsetValue = slide ? hiddenOffset(slide, axis) : 0;
  const hiddenProp = axis === "x" ? "x" : "y";
  const mm = gsap.matchMedia();

  mm.add("(prefers-reduced-motion: reduce)", () => {
    gsap.set(element, { autoAlpha: visible ? 1 : 0, [hiddenProp]: 0 });
  });

  mm.add("(prefers-reduced-motion: no-preference)", () => {
    gsap.to(element, {
      autoAlpha: visible ? 1 : 0,
      [hiddenProp]: visible ? 0 : hiddenOffsetValue,
      duration: SCROLL_OVERFLOW_FADE_DURATION_S,
      ease: SCROLL_OVERFLOW_FADE_EASE,
      overwrite: "auto",
    });
  });

  return () => {
    mm.revert();
  };
}

export function useGsapOverflowVisibility({
  visible,
  slide,
  axis = "x",
}: UseGsapOverflowVisibilityOptions): RefObject<HTMLDivElement | null> {
  const ref = useRef<HTMLDivElement>(null);
  const hiddenOffsetValue = slide ? hiddenOffset(slide, axis) : 0;

  useGSAP(
    () => {
      const element = ref.current;
      if (!element) {
        return undefined;
      }

      return animateOverflowVisibility(element, { visible, slide, axis });
    },
    {
      dependencies: [axis, hiddenOffsetValue, slide, visible],
      scope: ref,
      revertOnUpdate: true,
    },
  );

  return ref;
}
