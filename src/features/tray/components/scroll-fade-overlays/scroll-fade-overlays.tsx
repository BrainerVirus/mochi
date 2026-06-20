"use client";

import { ChevronDownIcon, ChevronUpIcon } from "lucide-react";
import { useEffect, useRef } from "react";

import { Button } from "@/components/ui/button";
import { TrayTabChevron } from "@/features/tray/components/tray-tab-chevron";
import { cn } from "@/lib/utils";

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
  const buttonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!visible) buttonRef.current?.blur();
  }, [visible]);

  return (
    <Button
      ref={buttonRef}
      type="button"
      disabled={!visible}
      variant="ghost"
      size="icon-xs"
      tabIndex={visible ? 0 : -1}
      aria-hidden={!visible}
      aria-label={isStart ? "Scroll up for more" : "Scroll down for more"}
      onClick={onCycle}
      className={cn(
        "pointer-events-auto absolute inset-x-0 z-20 mx-auto shrink-0 cursor-pointer rounded-full",
        "bg-background/35 text-muted-foreground shadow-none ring-0 backdrop-blur-[2px]",
        "hover:bg-background/50 hover:text-foreground",
        "transition-[opacity,translate] duration-200 ease-out motion-reduce:transition-none",
        isStart ? "top-0 mt-0.5" : "bottom-0 mb-0.5",
        visible ? "opacity-100" : "pointer-events-none opacity-0 disabled:opacity-0",
        !visible && (isStart ? "-translate-y-1" : "translate-y-1"),
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
