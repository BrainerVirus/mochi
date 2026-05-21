"use client";

import { useGSAP } from "@gsap/react";
import gsap from "gsap";
import { LayoutGridIcon } from "lucide-react";
import { useCallback, useRef, type RefObject } from "react";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { ProviderIcon } from "@/components/providers/provider-icon";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

gsap.registerPlugin(useGSAP);

/** Fixed row height shared with ScrollFadeRegion chevron overlay math. */
export const TRAY_SEGMENT_ROW_HEIGHT = "h-11" as const;

const INDICATOR_DURATION_S = 0.25;

const segmentItemClassName = cn(
  "relative z-10 h-full min-w-[4.75rem] max-w-[7.5rem] shrink-0 flex-none cursor-pointer flex-row items-center justify-center gap-1.5 rounded-none border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:bg-transparent hover:text-foreground",
  "data-[state=on]:bg-transparent data-[state=on]:font-semibold data-[state=on]:text-foreground data-[state=on]:shadow-none",
  "first:rounded-none last:rounded-none",
);

interface TraySegmentedControlProps {
  tabs: TrayPanelTab[];
  value: string;
  onValueChange: (value: string) => void;
}

function animateIndicator(
  track: HTMLElement,
  indicator: HTMLElement,
  activeItem: HTMLElement,
  instant: boolean,
) {
  const trackRect = track.getBoundingClientRect();
  const itemRect = activeItem.getBoundingClientRect();
  const x = itemRect.left - trackRect.left;
  const width = itemRect.width;

  const mm = gsap.matchMedia();

  mm.add("(prefers-reduced-motion: reduce)", () => {
    gsap.set(indicator, { x, width });
  });

  mm.add("(prefers-reduced-motion: no-preference)", () => {
    if (instant) {
      gsap.set(indicator, { x, width });
      return;
    }

    gsap.to(indicator, {
      x,
      width,
      duration: INDICATOR_DURATION_S,
      ease: "power2.out",
      overwrite: "auto",
    });
  });

  return () => {
    mm.revert();
  };
}

function TraySegmentItem({
  tab,
  setItemRef,
}: {
  tab: TrayPanelTab;
  setItemRef: (id: string, element: HTMLButtonElement | null) => void;
}) {
  return (
    <ToggleGroupItem
      ref={(element) => {
        setItemRef(tab.id, element);
      }}
      value={tab.id}
      aria-label={tab.label}
      className={segmentItemClassName}
    >
      {tab.id === "overview" ? (
        <LayoutGridIcon className="size-4 shrink-0 opacity-90" aria-hidden />
      ) : (
        <ProviderIcon provider={tab.id} />
      )}
      <span className="min-w-0 truncate text-xs">{tab.label}</span>
    </ToggleGroupItem>
  );
}

function useSlidingIndicator(
  trackRef: RefObject<HTMLDivElement | null>,
  value: string,
  tabCount: number,
  itemRefs: RefObject<Map<string, HTMLButtonElement>>,
) {
  const prevValueRef = useRef(value);

  useGSAP(
    () => {
      const track = trackRef.current;
      const indicator = track?.querySelector<HTMLElement>("[data-segment-indicator]");
      const activeItem = itemRefs.current?.get(value);

      if (!track || !indicator || !activeItem) {
        return undefined;
      }

      const valueChanged = prevValueRef.current !== value;
      prevValueRef.current = value;

      let revertMatchMedia = animateIndicator(track, indicator, activeItem, !valueChanged);

      const resizeObserver = new ResizeObserver(() => {
        const item = itemRefs.current?.get(value);
        if (!item) {
          return;
        }

        revertMatchMedia?.();
        revertMatchMedia = animateIndicator(track, indicator, item, true);
      });

      resizeObserver.observe(track);

      return () => {
        resizeObserver.disconnect();
        revertMatchMedia?.();
      };
    },
    { dependencies: [value, tabCount], scope: trackRef, revertOnUpdate: true },
  );
}

export function TraySegmentedControl({ tabs, value, onValueChange }: TraySegmentedControlProps) {
  const trackRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  const setItemRef = useCallback((id: string, element: HTMLButtonElement | null) => {
    if (element) {
      itemRefs.current.set(id, element);
      return;
    }

    itemRefs.current.delete(id);
  }, []);

  useSlidingIndicator(trackRef, value, tabs.length, itemRefs);

  return (
    <div
      ref={trackRef}
      className={cn(
        TRAY_SEGMENT_ROW_HEIGHT,
        "relative w-max min-w-full rounded-lg bg-muted/40 p-0.5",
      )}
    >
      <div
        data-segment-indicator
        aria-hidden
        className="pointer-events-none absolute top-0.5 bottom-0.5 left-0 z-0 rounded-md bg-background shadow-sm"
      />

      <ToggleGroup
        type="single"
        value={value}
        onValueChange={(next) => {
          if (next) {
            onValueChange(next);
          }
        }}
        spacing={0}
        variant="default"
        className="relative z-10 h-full w-max min-w-full flex-nowrap items-stretch justify-start gap-0 bg-transparent p-0 shadow-none"
      >
        {tabs.map((tab) => (
          <TraySegmentItem key={tab.id} tab={tab} setItemRef={setItemRef} />
        ))}
      </ToggleGroup>
    </div>
  );
}
