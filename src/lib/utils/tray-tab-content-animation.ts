import gsap from "gsap";

/** Matches {@link TRAY_PANEL_HEIGHT_DURATION_S} so content and panel height morph together. */
export const TRAY_TAB_CONTENT_DURATION_S = 0.28;
export const TRAY_TAB_CONTENT_STAGGER_S = 0.045;
export const TRAY_TAB_CONTENT_Y_PX = 6;
export const TRAY_TAB_CONTENT_EASE = "power2.out";

export const TRAY_TAB_ENTER_SELECTOR = "[data-tray-tab-enter]";

/**
 * Fade + slide in tab content sections when the active tab changes.
 * Runs simultaneously with the native tray height morph.
 */
export function runTrayTabContentEnterAnimation(container: HTMLElement): () => void {
  const sections = container.querySelectorAll<HTMLElement>(TRAY_TAB_ENTER_SELECTOR);
  if (sections.length === 0) {
    return () => {};
  }

  const mm = gsap.matchMedia();

  mm.add("(prefers-reduced-motion: reduce)", () => {
    gsap.set(sections, { autoAlpha: 1, y: 0 });
  });

  mm.add("(prefers-reduced-motion: no-preference)", () => {
    gsap.from(sections, {
      autoAlpha: 0,
      y: TRAY_TAB_CONTENT_Y_PX,
      duration: TRAY_TAB_CONTENT_DURATION_S,
      ease: TRAY_TAB_CONTENT_EASE,
      stagger: TRAY_TAB_CONTENT_STAGGER_S,
      overwrite: "auto",
    });
  });

  return () => {
    mm.revert();
  };
}

export function setTrayTabContentVisible(container: HTMLElement): void {
  const sections = container.querySelectorAll<HTMLElement>(TRAY_TAB_ENTER_SELECTOR);
  gsap.set(sections, { autoAlpha: 1, y: 0 });
}
