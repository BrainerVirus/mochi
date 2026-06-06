import { useCallback, useRef, type ReactNode } from "react";

import {
  cycleHorizontalScrollBackward,
  cycleHorizontalScrollForward,
  cycleVerticalScrollBackward,
  cycleVerticalScrollForward,
} from "@/components/tray/scroll-fade-cycle";
import { ScrollFadeEdgeOverlays } from "@/components/tray/scroll-fade-overlays";
import { useScrollOverflow } from "@/components/tray/use-scroll-overflow";
import { cn } from "@/lib/utils";

type ScrollFadeOrientation = "horizontal" | "vertical";

interface ScrollFadeRegionProps {
  orientation: ScrollFadeOrientation;
  children: ReactNode;
  className?: string;
  scrollClassName?: string;
  /** Tailwind height class matching the scroll row (e.g. tab list). Horizontal only. */
  rowHeightClassName?: string;
  /** Space reserved at the fade edge for the ghost control (px). */
  fadeInset?: number;
  controls?: "fade" | "none";
  /** Override default horizontal forward scroll (e.g. tab selection). */
  onCycleForward?: (scrollEl: HTMLDivElement) => void;
  /** Override default horizontal backward scroll (e.g. tab selection). */
  onCycleBackward?: (scrollEl: HTMLDivElement) => void;
}

function scrollFadeMaskClass(
  orientation: ScrollFadeOrientation,
  canScrollStart: boolean,
  canScrollEnd: boolean,
): string | undefined {
  if (orientation === "horizontal") {
    if (canScrollStart && canScrollEnd) {
      return "scroll-fade-mask-x-both";
    }
    if (canScrollStart) {
      return "scroll-fade-mask-x-start";
    }
    if (canScrollEnd) {
      return "scroll-fade-mask-x-end";
    }
    return undefined;
  }

  if (canScrollStart && canScrollEnd) {
    return "scroll-fade-mask-y-both";
  }
  if (canScrollStart) {
    return "scroll-fade-mask-y-start";
  }
  if (canScrollEnd) {
    return "scroll-fade-mask-y-end";
  }
  return undefined;
}

function useScrollCycle(
  scrollRef: React.RefObject<HTMLDivElement | null>,
  orientation: ScrollFadeOrientation,
  fadeInset: number,
  onCycleForward?: (scrollEl: HTMLDivElement) => void,
  onCycleBackward?: (scrollEl: HTMLDivElement) => void,
) {
  const cycleScrollForward = useCallback(() => {
    const el = scrollRef.current;
    if (!el) {
      return;
    }

    if (orientation === "horizontal") {
      if (onCycleForward) {
        onCycleForward(el);
        return;
      }

      cycleHorizontalScrollForward(el, fadeInset);
      return;
    }

    cycleVerticalScrollForward(el);
  }, [fadeInset, onCycleForward, orientation, scrollRef]);

  const cycleScrollBackward = useCallback(() => {
    const el = scrollRef.current;
    if (!el) {
      return;
    }

    if (orientation === "horizontal") {
      if (onCycleBackward) {
        onCycleBackward(el);
        return;
      }

      cycleHorizontalScrollBackward(el, fadeInset);
      return;
    }

    cycleVerticalScrollBackward(el);
  }, [fadeInset, onCycleBackward, orientation, scrollRef]);

  return { cycleScrollForward, cycleScrollBackward };
}

function ScrollFadeViewport({
  scrollRef,
  isHorizontal,
  canScrollStart,
  canScrollEnd,
  maskClass,
  scrollClassName,
  controls,
  onCycleForward,
  onCycleBackward,
  children,
}: {
  scrollRef: React.RefObject<HTMLDivElement | null>;
  isHorizontal: boolean;
  canScrollStart: boolean;
  canScrollEnd: boolean;
  maskClass: string | undefined;
  scrollClassName?: string;
  controls: "fade" | "none";
  onCycleForward: () => void;
  onCycleBackward: () => void;
  children: ReactNode;
}) {
  return (
    <div
      className={cn(
        "relative min-h-0 min-w-0 overflow-hidden",
        isHorizontal ? "h-full w-full" : "flex h-full min-h-0 flex-1 flex-col",
      )}
    >
      <div
        ref={scrollRef}
        className={cn(
          "overscroll-contain",
          controls !== "none" && "scrollbar-none",
          isHorizontal
            ? "h-full w-full overflow-x-auto overflow-y-hidden"
            : "min-h-0 flex-1 overflow-x-hidden overflow-y-auto",
          maskClass,
          scrollClassName,
        )}
      >
        {children}
      </div>

      {controls !== "none" ? (
        <ScrollFadeEdgeOverlays
          isHorizontal={isHorizontal}
          canScrollStart={canScrollStart}
          canScrollEnd={canScrollEnd}
          onCycleBackward={onCycleBackward}
          onCycleForward={onCycleForward}
        />
      ) : null}
    </div>
  );
}

export function ScrollFadeRegion({
  orientation,
  children,
  className,
  scrollClassName,
  rowHeightClassName,
  fadeInset = 40,
  controls = "fade",
  onCycleForward,
  onCycleBackward,
}: ScrollFadeRegionProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const { canScrollStart, canScrollEnd } = useScrollOverflow(scrollRef, orientation);
  const isHorizontal = orientation === "horizontal";
  const shouldUseMask = controls !== "none";
  const maskClass = shouldUseMask
    ? scrollFadeMaskClass(orientation, canScrollStart, canScrollEnd)
    : undefined;
  const { cycleScrollForward, cycleScrollBackward } = useScrollCycle(
    scrollRef,
    orientation,
    fadeInset,
    onCycleForward,
    onCycleBackward,
  );

  return (
    <div className={cn("relative min-w-0", isHorizontal && rowHeightClassName, className)}>
      <ScrollFadeViewport
        scrollRef={scrollRef}
        isHorizontal={isHorizontal}
        canScrollStart={canScrollStart}
        canScrollEnd={canScrollEnd}
        maskClass={maskClass}
        scrollClassName={scrollClassName}
        controls={controls}
        onCycleForward={cycleScrollForward}
        onCycleBackward={cycleScrollBackward}
      >
        {children}
      </ScrollFadeViewport>
    </div>
  );
}
