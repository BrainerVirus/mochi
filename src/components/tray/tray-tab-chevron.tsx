import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export type TrayTabChevronSide = "start" | "end";

interface TrayTabChevronProps {
  side: TrayTabChevronSide;
  visible: boolean;
  onCycle: () => void;
}

/** Full-height overlay column; icon centered on the tab strip, not the scroll viewport baseline. */
export function TrayTabChevron({ side, visible, onCycle }: TrayTabChevronProps) {
  const isStart = side === "start";

  return (
    <div
      className={cn(
        "pointer-events-none absolute top-1/2 z-20 flex size-8 -translate-y-1/2 items-center justify-center transition-[opacity,transform] duration-200 ease-out motion-reduce:transition-none",
        isStart ? "left-0" : "right-0",
        visible ? "translate-x-0 opacity-100" : "pointer-events-none opacity-0",
        isStart && !visible && "-translate-x-1",
        !isStart && !visible && "translate-x-1",
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
        className={cn(
          "pointer-events-auto shrink-0 cursor-pointer rounded-full text-muted-foreground hover:bg-muted/50 hover:text-foreground",
          !visible && "pointer-events-none",
        )}
      >
        {isStart ? (
          <ChevronLeftIcon className="size-3.5" aria-hidden />
        ) : (
          <ChevronRightIcon className="size-3.5" aria-hidden />
        )}
      </Button>
    </div>
  );
}
