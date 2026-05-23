"use client";

import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { ChevronDownIcon, ChevronUpIcon } from "lucide-react";
import { useRef } from "react";

import {
  animateOverflowVisibility,
  SCROLL_OVERFLOW_SLIDE_PX,
} from "@/components/tray/use-gsap-overflow-visibility";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

gsap.registerPlugin(useGSAP);

function ScrollFadeEdge({
  className,
  visible,
}: {
  className: string;
  visible: boolean;
}) {
  const edgeRef = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      const edge = edgeRef.current;
      if (!edge) {
        return undefined;
      }

      return animateOverflowVisibility(edge, { visible });
    },
    { dependencies: [visible], scope: edgeRef, revertOnUpdate: true },
  );

  return (
    <div
      ref={edgeRef}
      aria-hidden
      className={cn(className, "pointer-events-none", !visible && "opacity-0")}
    />
  );
}

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
      aria-hidden={!visible}
      tabIndex={visible ? 0 : -1}
      aria-label={isStart ? "Scroll up for more" : "Scroll down for more"}
      onClick={onCycle}
      className={cn(
        "pointer-events-auto absolute inset-x-0 z-20 mx-auto shrink-0 cursor-pointer rounded-full text-muted-foreground hover:bg-muted/50 hover:text-foreground",
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
        <ScrollFadeEdge className="scroll-fade-edge-left" visible={canScrollStart} />
        <ScrollFadeEdge className="scroll-fade-edge-right" visible={canScrollEnd} />
      </>
    );
  }

  return (
    <>
      <ScrollFadeEdge className="scroll-fade-edge-top" visible={canScrollStart} />
      <ScrollFadeEdge className="scroll-fade-edge-bottom" visible={canScrollEnd} />
      <ScrollFadeVerticalChevron side="start" visible={canScrollStart} onCycle={onCycleBackward} />
      <ScrollFadeVerticalChevron side="end" visible={canScrollEnd} onCycle={onCycleForward} />
    </>
  );
}
