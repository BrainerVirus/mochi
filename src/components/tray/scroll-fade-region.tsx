import { ChevronDownIcon, ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { useCallback, useRef, type ReactNode } from "react";

import {
  cycleHorizontalScrollBackward,
  cycleHorizontalScrollForward,
  cycleVerticalScroll,
} from "@/components/tray/scroll-fade-cycle";
import { useScrollOverflow } from "@/components/tray/use-scroll-overflow";
import { Button } from "@/components/ui/button";
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

type ScrollFadeDirection = "forward" | "backward";

function ScrollFadeGhostButton({
  orientation,
  direction = "forward",
  visible,
  onCycle,
  className,
}: {
  orientation: ScrollFadeOrientation;
  direction?: ScrollFadeDirection;
  visible: boolean;
  onCycle: () => void;
  className?: string;
}) {
  const isHorizontal = orientation === "horizontal";
  const isBackward = direction === "backward";

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-xs"
      aria-hidden={!visible}
      tabIndex={visible ? 0 : -1}
      aria-label={
        isHorizontal
          ? isBackward
            ? "Show previous tabs"
            : "Show more tabs"
          : "Scroll down for more"
      }
      onClick={onCycle}
      className={cn(
        "pointer-events-auto z-20 shrink-0 cursor-pointer rounded-full text-muted-foreground transition-[opacity,transform] duration-200 ease-out hover:bg-muted/50 hover:text-foreground",
        isHorizontal
          ? cn(
              "absolute top-1/2 -translate-y-1/2",
              isBackward
                ? visible
                  ? "left-0 translate-x-0 opacity-100"
                  : "left-0 -translate-x-1 opacity-0 pointer-events-none"
                : visible
                  ? "right-0 translate-x-0 opacity-100"
                  : "right-0 translate-x-1 opacity-0 pointer-events-none",
            )
          : cn(
              "absolute inset-x-0 bottom-0 z-20 mx-auto mb-0.5",
              visible ? "translate-y-0 opacity-100" : "translate-y-1 opacity-0 pointer-events-none",
            ),
        className,
      )}
    >
      {isHorizontal ? (
        isBackward ? (
          <ChevronLeftIcon className="size-3.5" />
        ) : (
          <ChevronRightIcon className="size-3.5" />
        )
      ) : (
        <ChevronDownIcon className="size-3.5" />
      )}
    </Button>
  );
}

function useScrollCycle(
  scrollRef: React.RefObject<HTMLDivElement | null>,
  orientation: ScrollFadeOrientation,
  fadeInset: number,
) {
  const cycleScrollForward = useCallback(() => {
    const el = scrollRef.current;
    if (!el) {
      return;
    }

    if (orientation === "horizontal") {
      cycleHorizontalScrollForward(el, fadeInset);
      return;
    }

    cycleVerticalScroll(el);
  }, [fadeInset, orientation, scrollRef]);

  const cycleScrollBackward = useCallback(() => {
    const el = scrollRef.current;
    if (!el || orientation !== "horizontal") {
      return;
    }

    cycleHorizontalScrollBackward(el, fadeInset);
  }, [fadeInset, orientation, scrollRef]);

  return { cycleScrollForward, cycleScrollBackward };
}

function ScrollFadeViewport({
  scrollRef,
  isHorizontal,
  canScrollStart,
  canScrollEnd,
  orientation,
  maskClass,
  scrollClassName,
  onCycleForward,
  onCycleBackward,
  children,
}: {
  scrollRef: React.RefObject<HTMLDivElement | null>;
  isHorizontal: boolean;
  canScrollStart: boolean;
  canScrollEnd: boolean;
  orientation: ScrollFadeOrientation;
  maskClass: string | undefined;
  scrollClassName?: string;
  onCycleForward: () => void;
  onCycleBackward: () => void;
  children: ReactNode;
}) {
  return (
    <div className={cn("relative min-w-0", isHorizontal && "h-full w-full")}>
      <div
        ref={scrollRef}
        className={cn(
          "scrollbar-none overscroll-contain",
          isHorizontal ? "h-full overflow-x-auto overflow-y-hidden" : "overflow-x-hidden overflow-y-auto",
          maskClass,
          scrollClassName,
        )}
      >
        {children}
      </div>

      {canScrollStart && isHorizontal ? (
        <div aria-hidden className="scroll-fade-edge-left" />
      ) : null}
      {canScrollEnd && isHorizontal ? (
        <div aria-hidden className="scroll-fade-edge-right" />
      ) : null}
      {canScrollEnd && !isHorizontal ? (
        <>
          <div aria-hidden className="scroll-fade-edge-bottom" />
          <ScrollFadeGhostButton
            orientation={orientation}
            visible={canScrollEnd}
            onCycle={onCycleForward}
          />
        </>
      ) : null}

      {isHorizontal ? (
        <>
          <ScrollFadeGhostButton
            orientation={orientation}
            direction="backward"
            visible={canScrollStart}
            onCycle={onCycleBackward}
          />
          <ScrollFadeGhostButton
            orientation={orientation}
            direction="forward"
            visible={canScrollEnd}
            onCycle={onCycleForward}
          />
        </>
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
}: ScrollFadeRegionProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const { canScrollStart, canScrollEnd } = useScrollOverflow(scrollRef, orientation);
  const isHorizontal = orientation === "horizontal";
  const maskClass = scrollFadeMaskClass(orientation, canScrollStart, canScrollEnd);
  const { cycleScrollForward, cycleScrollBackward } = useScrollCycle(
    scrollRef,
    orientation,
    fadeInset,
  );

  return (
    <div
      className={cn(
        "relative",
        isHorizontal && rowHeightClassName,
        className,
      )}
    >
      <ScrollFadeViewport
        scrollRef={scrollRef}
        isHorizontal={isHorizontal}
        canScrollStart={canScrollStart}
        canScrollEnd={canScrollEnd}
        orientation={orientation}
        maskClass={maskClass}
        scrollClassName={scrollClassName}
        onCycleForward={cycleScrollForward}
        onCycleBackward={cycleScrollBackward}
      >
        {children}
      </ScrollFadeViewport>
    </div>
  );
}
