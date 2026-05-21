import { useCallback, useEffect, useState, type RefObject } from "react";

type ScrollFadeOrientation = "horizontal" | "vertical";

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

    if (orientation === "horizontal") {
      setCanScrollStart(el.scrollLeft > 1);
      setCanScrollEnd(el.scrollLeft + el.clientWidth < el.scrollWidth - 1);
      return;
    }

    setCanScrollStart(false);
    setCanScrollEnd(el.scrollTop + el.clientHeight < el.scrollHeight - 1);
  }, [orientation, scrollRef]);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) {
      return () => {};
    }

    checkOverflow();
    el.addEventListener("scroll", checkOverflow, { passive: true });
    const observer = new ResizeObserver(checkOverflow);
    observer.observe(el);

    return () => {
      el.removeEventListener("scroll", checkOverflow);
      observer.disconnect();
    };
  }, [checkOverflow, scrollRef]);

  return { canScrollStart, canScrollEnd };
}
