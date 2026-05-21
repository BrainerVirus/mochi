import { ChevronDownIcon } from "lucide-react";
import { useCallback, useRef, type ReactNode } from "react";

import {
  cycleHorizontalScrollBackward,
  cycleHorizontalScrollForward,
  cycleVerticalScroll,
} from "@/components/tray/scroll-fade-cycle";
import { TrayTabChevron } from "@/components/tray/tray-tab-chevron";
import { useScrollOverflow } from "@/components/tray/use-scroll-overflow";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type ScrollFadeOrientation = "horizontal" | "vertical";

/** Matches overlay chevron column width (w-8). */
const HORIZONTAL_CHEVRON_INSET_START = "pl-8";
const HORIZONTAL_CHEVRON_INSET_END = "pr-8";

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

function ScrollFadeVerticalChevron({
  visible,
  onCycle,
}: {
  visible: boolean;
  onCycle: () => void;
}) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-xs"
      aria-hidden={!visible}
      tabIndex={visible ? 0 : -1}
      aria-label="Scroll down for more"
      onClick={onCycle}
      className={cn(
        "pointer-events-auto absolute inset-x-0 bottom-0 z-20 mx-auto mb-0.5 shrink-0 cursor-pointer rounded-full text-muted-foreground transition-[opacity,transform] duration-200 ease-out hover:bg-muted/50 hover:text-foreground motion-reduce:transition-none",
        visible ? "translate-y-0 opacity-100" : "pointer-events-none translate-y-1 opacity-0",
      )}
    >
      <ChevronDownIcon className="size-3.5" />
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
  maskClass,
  scrollClassName,
  onCycleForward,
  children,
}: {
  scrollRef: React.RefObject<HTMLDivElement | null>;
  isHorizontal: boolean;
  canScrollStart: boolean;
  canScrollEnd: boolean;
  maskClass: string | undefined;
  scrollClassName?: string;
  onCycleForward: () => void;
  children: ReactNode;
}) {
  return (
    <div className={cn("relative min-h-0 min-w-0", isHorizontal && "h-full w-full")}>
      <div
        ref={scrollRef}
        className={cn(
          "scrollbar-none overscroll-contain",
          isHorizontal ? "h-full overflow-x-auto overflow-y-hidden" : "overflow-x-hidden overflow-y-auto",
          isHorizontal && canScrollStart && HORIZONTAL_CHEVRON_INSET_START,
          isHorizontal && canScrollEnd && HORIZONTAL_CHEVRON_INSET_END,
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
          <ScrollFadeVerticalChevron visible={canScrollEnd} onCycle={onCycleForward} />
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
        maskClass={maskClass}
        scrollClassName={scrollClassName}
        onCycleForward={cycleScrollForward}
      >
        {children}
      </ScrollFadeViewport>

      {isHorizontal ? (
        <>
          <TrayTabChevron
            side="start"
            visible={canScrollStart}
            onCycle={cycleScrollBackward}
          />
          <TrayTabChevron side="end" visible={canScrollEnd} onCycle={cycleScrollForward} />
        </>
      ) : null}
    </div>
  );
}
