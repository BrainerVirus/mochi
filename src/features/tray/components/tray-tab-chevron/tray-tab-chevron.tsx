"use client";

import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { getTrayTabChevronButtonClassName } from "@/features/tray/components/tray-tab-chevron-class-name";
import { cn } from "@/lib/utils";

interface TrayTabChevronProps {
  side: "start" | "end";
  visible: boolean;
  onCycle: () => void;
}

/** Full-height overlay column; icon centered on the tab strip, not the scroll viewport baseline. */
export function TrayTabChevron({ side, visible, onCycle }: TrayTabChevronProps) {
  const isStart = side === "start";

  return (
    <div
      className={cn(
        "pointer-events-none absolute inset-y-0 z-30 flex w-8 items-center justify-center",
        "transition-[opacity,transform] duration-200 ease-out motion-reduce:transition-none",
        isStart ? "left-0" : "right-0",
        visible ? "opacity-100" : "opacity-0",
        !visible && (isStart ? "-translate-x-1" : "translate-x-1"),
      )}
      aria-hidden={!visible}
    >
      <Button
        type="button"
        variant="ghost"
        size="icon-xs"
        tabIndex={visible ? 0 : -1}
        aria-label={isStart ? "Show previous tabs" : "Show more tabs"}
        onClick={onCycle}
        className={getTrayTabChevronButtonClassName(visible)}
      >
        {isStart ? (
          <ChevronLeftIcon className="size-4 stroke-[3]" aria-hidden />
        ) : (
          <ChevronRightIcon className="size-4 stroke-[3]" aria-hidden />
        )}
      </Button>
    </div>
  );
}
