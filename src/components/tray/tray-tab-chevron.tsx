import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { useRef } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

gsap.registerPlugin(useGSAP);

const CHEVRON_SLIDE_PX = 4;
const CHEVRON_DURATION_S = 0.2;

export type TrayTabChevronSide = "start" | "end";

interface TrayTabChevronProps {
  side: TrayTabChevronSide;
  visible: boolean;
  onCycle: () => void;
}

/** Full-height overlay column; icon centered on the tab strip, not the scroll viewport baseline. */
export function TrayTabChevron({ side, visible, onCycle }: TrayTabChevronProps) {
  const columnRef = useRef<HTMLDivElement>(null);
  const isStart = side === "start";
  const hiddenX = isStart ? -CHEVRON_SLIDE_PX : CHEVRON_SLIDE_PX;

  useGSAP(
    () => {
      const column = columnRef.current;
      if (!column) {
        return undefined;
      }

      const mm = gsap.matchMedia();

      mm.add("(prefers-reduced-motion: reduce)", () => {
        gsap.set(column, { autoAlpha: visible ? 1 : 0, x: 0 });
      });

      mm.add("(prefers-reduced-motion: no-preference)", () => {
        gsap.to(column, {
          autoAlpha: visible ? 1 : 0,
          x: visible ? 0 : hiddenX,
          duration: CHEVRON_DURATION_S,
          ease: "power2.out",
          overwrite: "auto",
        });
      });

      return () => {
        mm.revert();
      };
    },
    { dependencies: [hiddenX, visible], scope: columnRef, revertOnUpdate: true },
  );

  return (
    <div
      ref={columnRef}
      className={cn(
        "pointer-events-none absolute top-1/2 z-20 flex size-8 -translate-y-1/2 items-center justify-center",
        isStart ? "left-0" : "right-0",
        !visible && "invisible opacity-0",
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
