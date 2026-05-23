import { ChevronDownIcon, ChevronUpIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export const SCROLL_FADE_EDGE_TRANSITION =
  "pointer-events-none transition-opacity duration-200 ease-out motion-reduce:transition-none";

function ScrollFadeVerticalChevron({
  side,
  visible,
  onCycle,
}: {
  side: "start" | "end";
  visible: boolean;
  onCycle: () => void;
}) {
  const isStart = side === "start";

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-xs"
      aria-hidden={!visible}
      tabIndex={visible ? 0 : -1}
      aria-label={isStart ? "Scroll up for more" : "Scroll down for more"}
      onClick={onCycle}
      className={cn(
        "pointer-events-auto absolute inset-x-0 z-20 mx-auto shrink-0 cursor-pointer rounded-full text-muted-foreground transition-[opacity,transform] duration-200 ease-out hover:bg-muted/50 hover:text-foreground motion-reduce:transition-none",
        isStart ? "top-0 mt-0.5" : "bottom-0 mb-0.5",
        visible ? "translate-y-0 opacity-100" : "pointer-events-none opacity-0",
        isStart && !visible && "-translate-y-1",
        !isStart && !visible && "translate-y-1",
      )}
    >
      {isStart ? <ChevronUpIcon className="size-3.5" /> : <ChevronDownIcon className="size-3.5" />}
    </Button>
  );
}

export function ScrollFadeEdgeOverlays({
  isHorizontal,
  canScrollStart,
  canScrollEnd,
  onCycleBackward,
  onCycleForward,
}: {
  isHorizontal: boolean;
  canScrollStart: boolean;
  canScrollEnd: boolean;
  onCycleBackward: () => void;
  onCycleForward: () => void;
}) {
  if (isHorizontal) {
    return (
      <>
        <div
          aria-hidden
          className={cn(
            "scroll-fade-edge-left",
            SCROLL_FADE_EDGE_TRANSITION,
            canScrollStart ? "opacity-100" : "opacity-0",
          )}
        />
        <div
          aria-hidden
          className={cn(
            "scroll-fade-edge-right",
            SCROLL_FADE_EDGE_TRANSITION,
            canScrollEnd ? "opacity-100" : "opacity-0",
          )}
        />
      </>
    );
  }

  return (
    <>
      <div
        aria-hidden
        className={cn(
          "scroll-fade-edge-top",
          SCROLL_FADE_EDGE_TRANSITION,
          canScrollStart ? "opacity-100" : "opacity-0",
        )}
      />
      <div
        aria-hidden
        className={cn(
          "scroll-fade-edge-bottom",
          SCROLL_FADE_EDGE_TRANSITION,
          canScrollEnd ? "opacity-100" : "opacity-0",
        )}
      />
      <ScrollFadeVerticalChevron side="start" visible={canScrollStart} onCycle={onCycleBackward} />
      <ScrollFadeVerticalChevron side="end" visible={canScrollEnd} onCycle={onCycleForward} />
    </>
  );
}
