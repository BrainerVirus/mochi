import { useCallback, useEffect, useRef, useState, type RefObject } from "react";

type ScrollFadeOrientation = "horizontal" | "vertical";

interface ScrollOverflowMetrics {
  scrollTop: number;
  scrollLeft: number;
  clientHeight: number;
  scrollHeight: number;
  clientWidth: number;
  scrollWidth: number;
}

export const SCROLL_OVERFLOW_SHOW_THRESHOLD = 4;
export const SCROLL_OVERFLOW_HIDE_THRESHOLD = 1;

export function measureScrollOverflow(
  el: ScrollOverflowMetrics,
  orientation: ScrollFadeOrientation,
): { canScrollStart: boolean; canScrollEnd: boolean } {
  if (orientation === "horizontal") {
    return {
      canScrollStart: el.scrollLeft > SCROLL_OVERFLOW_HIDE_THRESHOLD,
      canScrollEnd:
        el.scrollLeft + el.clientWidth < el.scrollWidth - SCROLL_OVERFLOW_HIDE_THRESHOLD,
    };
  }

  return {
    canScrollStart: el.scrollTop > SCROLL_OVERFLOW_HIDE_THRESHOLD,
    canScrollEnd: el.scrollTop + el.clientHeight < el.scrollHeight - SCROLL_OVERFLOW_HIDE_THRESHOLD,
  };
}

export function measureScrollOverflowWithHysteresis(
  el: ScrollOverflowMetrics,
  orientation: ScrollFadeOrientation,
  previous: { canScrollStart: boolean; canScrollEnd: boolean },
): { canScrollStart: boolean; canScrollEnd: boolean } {
  if (orientation === "horizontal") {
    const maxScrollLeft = Math.max(0, el.scrollWidth - el.clientWidth);

    return {
      canScrollStart: previous.canScrollStart
        ? el.scrollLeft > SCROLL_OVERFLOW_HIDE_THRESHOLD
        : el.scrollLeft > SCROLL_OVERFLOW_SHOW_THRESHOLD,
      canScrollEnd: previous.canScrollEnd
        ? el.scrollLeft < maxScrollLeft - SCROLL_OVERFLOW_HIDE_THRESHOLD
        : el.scrollLeft < maxScrollLeft - SCROLL_OVERFLOW_SHOW_THRESHOLD,
    };
  }

  const maxScrollTop = Math.max(0, el.scrollHeight - el.clientHeight);

  return {
    canScrollStart: previous.canScrollStart
      ? el.scrollTop > SCROLL_OVERFLOW_HIDE_THRESHOLD
      : el.scrollTop > SCROLL_OVERFLOW_SHOW_THRESHOLD,
    canScrollEnd: previous.canScrollEnd
      ? el.scrollTop < maxScrollTop - SCROLL_OVERFLOW_HIDE_THRESHOLD
      : el.scrollTop < maxScrollTop - SCROLL_OVERFLOW_SHOW_THRESHOLD,
  };
}

export function useScrollOverflow(
  scrollRef: RefObject<HTMLDivElement | null>,
  orientation: ScrollFadeOrientation,
) {
  const [canScrollStart, setCanScrollStart] = useState(false);
  const [canScrollEnd, setCanScrollEnd] = useState(false);
  const overflowRef = useRef({ canScrollStart: false, canScrollEnd: false });

  const checkOverflow = useCallback(() => {
    const el = scrollRef.current;
    if (!el) {
      return;
    }

    const next = measureScrollOverflowWithHysteresis(el, orientation, overflowRef.current);
    overflowRef.current = next;
    setCanScrollStart(next.canScrollStart);
    setCanScrollEnd(next.canScrollEnd);
  }, [orientation, scrollRef]);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) {
      return () => {};
    }

    checkOverflow();
    el.addEventListener("scroll", checkOverflow, { passive: true });

    const resizeObserver = new ResizeObserver(checkOverflow);
    resizeObserver.observe(el);

    const observeScrollContent = () => {
      for (const child of el.children) {
        if (child instanceof HTMLElement) {
          resizeObserver.observe(child);
        }
      }
    };

    observeScrollContent();

    const mutationObserver = new MutationObserver(() => {
      observeScrollContent();
      checkOverflow();
    });
    mutationObserver.observe(el, { childList: true, subtree: true });

    return () => {
      el.removeEventListener("scroll", checkOverflow);
      resizeObserver.disconnect();
      mutationObserver.disconnect();
    };
  }, [checkOverflow, scrollRef]);

  return { canScrollStart, canScrollEnd };
}
