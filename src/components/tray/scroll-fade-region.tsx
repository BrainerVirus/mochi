import { ChevronDownIcon, ChevronRightIcon } from "lucide-react";
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type ScrollFadeOrientation = "horizontal" | "vertical";

interface ScrollFadeRegionProps {
  orientation: ScrollFadeOrientation;
  children: ReactNode;
  className?: string;
  scrollClassName?: string;
  /** Space reserved at the fade edge for the ghost control (px). */
  fadeInset?: number;
}

function cycleHorizontalScroll(container: HTMLDivElement, fadeInset: number) {
  const triggers = [...container.querySelectorAll<HTMLElement>('[data-slot="tabs-trigger"]')];
  const visibleRight = container.scrollLeft + container.clientWidth - fadeInset;

  for (const trigger of triggers) {
    if (trigger.offsetLeft + trigger.offsetWidth > visibleRight + 1) {
      container.scrollTo({ left: trigger.offsetLeft, behavior: "smooth" });
      return;
    }
  }

  container.scrollTo({ left: 0, behavior: "smooth" });
}

function cycleVerticalScroll(container: HTMLDivElement) {
  const step = container.clientHeight * 0.75;
  const maxScroll = container.scrollHeight - container.clientHeight;
  const next = container.scrollTop + step;

  if (next >= maxScroll - 1) {
    container.scrollTo({ top: 0, behavior: "smooth" });
    return;
  }

  container.scrollTo({ top: next, behavior: "smooth" });
}

function useScrollOverflow(
  scrollRef: React.RefObject<HTMLDivElement | null>,
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

function scrollFadeMaskClass(
  orientation: ScrollFadeOrientation,
  canScrollStart: boolean,
  canScrollEnd: boolean,
): string | undefined {
  if (!canScrollEnd) {
    return undefined;
  }

  if (orientation === "horizontal") {
    return canScrollStart ? "scroll-fade-mask-x-both" : "scroll-fade-mask-x-end";
  }

  return "scroll-fade-mask-y-end";
}

function ScrollFadeGhostButton({
  orientation,
  onCycle,
}: {
  orientation: ScrollFadeOrientation;
  onCycle: () => void;
}) {
  const isHorizontal = orientation === "horizontal";

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-xs"
      aria-label={isHorizontal ? "Show more tabs" : "Scroll down for more"}
      onClick={onCycle}
      className={cn(
        "pointer-events-auto absolute z-20 size-7 cursor-pointer rounded-full text-muted-foreground hover:bg-muted/50 hover:text-foreground",
        isHorizontal ? "inset-y-0 right-0 my-auto mr-0.5" : "inset-x-0 bottom-0 mx-auto mb-0.5",
      )}
    >
      {isHorizontal ? (
        <ChevronRightIcon className="size-3.5" />
      ) : (
        <ChevronDownIcon className="size-3.5" />
      )}
    </Button>
  );
}

export function ScrollFadeRegion({
  orientation,
  children,
  className,
  scrollClassName,
  fadeInset = 40,
}: ScrollFadeRegionProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const { canScrollStart, canScrollEnd } = useScrollOverflow(scrollRef, orientation);
  const isHorizontal = orientation === "horizontal";
  const maskClass = scrollFadeMaskClass(orientation, canScrollStart, canScrollEnd);

  const cycleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) {
      return;
    }

    if (orientation === "horizontal") {
      cycleHorizontalScroll(el, fadeInset);
      return;
    }

    cycleVerticalScroll(el);
  }, [fadeInset, orientation]);

  return (
    <div className={cn("relative", className)}>
      <div
        ref={scrollRef}
        className={cn(
          "scrollbar-none overscroll-contain",
          isHorizontal ? "overflow-x-auto overflow-y-hidden" : "overflow-x-hidden overflow-y-auto",
          maskClass,
          scrollClassName,
        )}
      >
        {children}
      </div>

      {canScrollStart && isHorizontal ? <div aria-hidden className="scroll-fade-edge-left" /> : null}
      {canScrollEnd ? (
        <>
          <div
            aria-hidden
            className={isHorizontal ? "scroll-fade-edge-right" : "scroll-fade-edge-bottom"}
          />
          <ScrollFadeGhostButton orientation={orientation} onCycle={cycleScroll} />
        </>
      ) : null}
    </div>
  );
}
