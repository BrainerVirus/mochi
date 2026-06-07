"use client";

import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { ChevronDownIcon, ChevronUpIcon } from "lucide-react";
import { useRef } from "react";

import { Button } from "@/components/ui/button";
import { TrayTabChevron } from "@/features/tray/components/tray-tab-chevron";
import {
  animateOverflowVisibility,
  SCROLL_OVERFLOW_SLIDE_PX,
} from "@/features/tray/components/use-gsap-overflow-visibility";
import { cn } from "@/lib/utils";

gsap.registerPlugin(useGSAP);

function ScrollFadeVerticalChevron({
  side,
  visible,
  onCycle,
}: {
  side: "start" | "end";
  visible: boolean;
  onCycle: () => void;
}) {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const isStart = side === "start";
  const hiddenY = isStart ? -SCROLL_OVERFLOW_SLIDE_PX : SCROLL_OVERFLOW_SLIDE_PX;

  useGSAP(
    () => {
      const button = buttonRef.current;
      if (!button) {
        return undefined;
      }

      return animateOverflowVisibility(button, { visible, slide: side, axis: "y" });
    },
    { dependencies: [hiddenY, side, visible], scope: buttonRef, revertOnUpdate: true },
  );

  return (
    <Button
      ref={buttonRef}
      type="button"
      variant="ghost"
      size="icon-xs"
      tabIndex={visible ? 0 : -1}
      aria-label={isStart ? "Scroll up for more" : "Scroll down for more"}
      onClick={onCycle}
      className={cn(
        "pointer-events-auto absolute inset-x-0 z-20 mx-auto shrink-0 cursor-pointer rounded-full",
        "bg-background/35 text-muted-foreground shadow-none ring-0 backdrop-blur-[2px]",
        "hover:bg-background/50 hover:text-foreground",
        isStart ? "top-0 mt-0.5" : "bottom-0 mb-0.5",
        !visible && "pointer-events-none opacity-0",
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
        <TrayTabChevron side="start" visible={canScrollStart} onCycle={onCycleBackward} />
        <TrayTabChevron side="end" visible={canScrollEnd} onCycle={onCycleForward} />
      </>
    );
  }

  return (
    <>
      <ScrollFadeVerticalChevron side="start" visible={canScrollStart} onCycle={onCycleBackward} />
      <ScrollFadeVerticalChevron side="end" visible={canScrollEnd} onCycle={onCycleForward} />
    </>
  );
}
