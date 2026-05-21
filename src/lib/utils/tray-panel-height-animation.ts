import gsap from "gsap";

import {
  clampTrayPanelHeight,
  measureTrayPanelLayoutHeight,
} from "@/lib/utils/tray-panel-layout";

export const TRAY_PANEL_HEIGHT_DURATION_S = 0.32;
export const TRAY_PANEL_HEIGHT_EASE = "power2.out";

export function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function resolveTrayPanelHeight(layoutEl: HTMLElement): number {
  const viewportHeight = window.screen.availHeight;
  const layoutHeight = measureTrayPanelLayoutHeight(layoutEl);
  return clampTrayPanelHeight(layoutHeight, viewportHeight);
}

interface AnimateTrayPanelHeightOptions {
  from: number;
  to: number;
  onUpdate: (height: number) => void;
  onComplete?: () => void;
}

export function animateTrayPanelHeightTo({
  from,
  to,
  onUpdate,
  onComplete,
}: AnimateTrayPanelHeightOptions): gsap.core.Tween {
  const proxy = { height: from };

  return gsap.to(proxy, {
    height: to,
    duration: TRAY_PANEL_HEIGHT_DURATION_S,
    ease: TRAY_PANEL_HEIGHT_EASE,
    overwrite: "auto",
    onUpdate: () => {
      onUpdate(Math.round(proxy.height));
    },
    onComplete,
  });
}
