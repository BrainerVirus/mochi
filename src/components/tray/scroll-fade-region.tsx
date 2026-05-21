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
  onCycle,
  className,
}: {
  orientation: ScrollFadeOrientation;
  direction?: ScrollFadeDirection;
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
      aria-label={
        isHorizontal
          ? isBackward
            ? "Show previous tabs"
            : "Show more tabs"
          : "Scroll down for more"
      }
      onClick={onCycle}
      className={cn(
        "pointer-events-auto z-20 shrink-0 cursor-pointer rounded-full text-muted-foreground hover:bg-muted/50 hover:text-foreground",
        isHorizontal ? undefined : "absolute inset-x-0 bottom-0 z-20 mx-auto mb-0.5",
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

function HorizontalChevronColumn({ children }: { children: ReactNode }) {
  return (
    <div className="flex shrink-0 items-center justify-center self-stretch px-0.5">
      {children}
    </div>
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
  children: ReactNode;
}) {
  return (
    <div className={cn("relative min-w-0", isHorizontal && "flex-1")}>
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

      {canScrollStart && isHorizontal ? (
        <div aria-hidden className="scroll-fade-edge-left" />
      ) : null}
      {canScrollEnd && isHorizontal ? (
        <div aria-hidden className="scroll-fade-edge-right" />
      ) : null}
      {canScrollEnd && !isHorizontal ? (
        <>
          <div aria-hidden className="scroll-fade-edge-bottom" />
          <ScrollFadeGhostButton orientation={orientation} onCycle={onCycleForward} />
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
    <div className={cn("relative", isHorizontal && "flex items-stretch", className)}>
      {canScrollStart && isHorizontal ? (
        <HorizontalChevronColumn>
          <ScrollFadeGhostButton
            orientation={orientation}
            direction="backward"
            onCycle={cycleScrollBackward}
          />
        </HorizontalChevronColumn>
      ) : null}

      <ScrollFadeViewport
        scrollRef={scrollRef}
        isHorizontal={isHorizontal}
        canScrollStart={canScrollStart}
        canScrollEnd={canScrollEnd}
        orientation={orientation}
        maskClass={maskClass}
        scrollClassName={scrollClassName}
        onCycleForward={cycleScrollForward}
      >
        {children}
      </ScrollFadeViewport>

      {canScrollEnd && isHorizontal ? (
        <HorizontalChevronColumn>
          <ScrollFadeGhostButton
            orientation={orientation}
            direction="forward"
            onCycle={cycleScrollForward}
          />
        </HorizontalChevronColumn>
      ) : null}
    </div>
  );
}
