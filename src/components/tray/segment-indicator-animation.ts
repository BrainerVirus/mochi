import gsap from "gsap";

export function isActiveIndicatorAnimating(indicator: HTMLElement | null): boolean {
  return indicator !== null && gsap.isTweening(indicator);
}
