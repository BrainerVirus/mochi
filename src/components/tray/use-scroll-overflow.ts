import { useCallback, useEffect, useState, type RefObject } from "react";

type ScrollFadeOrientation = "horizontal" | "vertical";

interface ScrollOverflowMetrics {
  scrollTop: number;
  scrollLeft: number;
  clientHeight: number;
  scrollHeight: number;
  clientWidth: number;
  scrollWidth: number;
}

export function measureScrollOverflow(
  el: ScrollOverflowMetrics,
  orientation: ScrollFadeOrientation,
): { canScrollStart: boolean; canScrollEnd: boolean } {
  if (orientation === "horizontal") {
    return {
      canScrollStart: el.scrollLeft > 1,
      canScrollEnd: el.scrollLeft + el.clientWidth < el.scrollWidth - 1,
    };
  }

  return {
    canScrollStart: el.scrollTop > 1,
    canScrollEnd: el.scrollTop + el.clientHeight < el.scrollHeight - 1,
  };
}

export function useScrollOverflow(
  scrollRef: RefObject<HTMLDivElement | null>,
  orientation: ScrollFadeOrientation,
) {
  const [canScrollStart, setCanScrollStart] = useState(false);
  const [canScrollEnd, setCanScrollEnd] = useState(false);

  const checkOverflow = useCallback(() => {
    const el = scrollRef.current;
    if (!el) {
      return;
    }

    const next = measureScrollOverflow(el, orientation);
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
