"use client";

import { useCallback, useRef, type RefObject } from "react";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { TraySegmentItem } from "@/components/tray/tray-segment-item";
import { useTraySegmentIndicators } from "@/components/tray/use-tray-segment-indicators";
import { ToggleGroup } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

/** Fixed row height shared with ScrollFadeRegion chevron overlay math. */
export const TRAY_SEGMENT_ROW_HEIGHT = "h-11" as const;

const indicatorLayerClassName =
  "pointer-events-none absolute inset-y-0.5 left-0 rounded-md will-change-[transform,width]";

interface TraySegmentedControlProps {
  tabs: TrayPanelTab[];
  value: string;
  onValueChange: (value: string) => void;
}

function TraySegmentIndicators({
  hoverIndicatorRef,
  activeIndicatorRef,
}: {
  hoverIndicatorRef: RefObject<HTMLDivElement | null>;
  activeIndicatorRef: RefObject<HTMLDivElement | null>;
}) {
  return (
    <>
      <div
        ref={hoverIndicatorRef}
        data-segment-hover-indicator
        aria-hidden
        className={cn(indicatorLayerClassName, "z-[1] invisible bg-[var(--tray-segment-hover)]")}
        style={{ width: 0 }}
      />
      <div
        ref={activeIndicatorRef}
        data-segment-indicator
        aria-hidden
        className={cn(
          indicatorLayerClassName,
          "z-[2] invisible bg-[var(--tray-segment-active)] shadow-sm ring-1 ring-[var(--tray-panel-stroke)]",
        )}
        style={{ width: 0 }}
      />
    </>
  );
}

export function TraySegmentedControl({ tabs, value, onValueChange }: TraySegmentedControlProps) {
  const trackRef = useRef<HTMLDivElement>(null);
  const activeIndicatorRef = useRef<HTMLDivElement>(null);
  const hoverIndicatorRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  const setItemRef = useCallback((id: string, element: HTMLButtonElement | null) => {
    if (element) {
      itemRefs.current.set(id, element);
      return;
    }
    itemRefs.current.delete(id);
  }, []);

  const {
    syncHoverIndicator,
    handleHoverEnd,
    handlePointerDown,
    handlePointerUp,
    handleSegmentValueChange,
  } = useTraySegmentIndicators(
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    value,
    tabs.length,
    itemRefs,
  );

  const handleValueChange = useCallback(
    (next: string) => {
      if (next) {
        handleSegmentValueChange(next, onValueChange);
      }
    },
    [handleSegmentValueChange, onValueChange],
  );

  return (
    <div
      ref={trackRef}
      className={cn(
        TRAY_SEGMENT_ROW_HEIGHT,
        "relative isolate w-max min-w-full rounded-lg bg-[var(--tray-segment-track)] p-0.5",
      )}
    >
      <TraySegmentIndicators
        hoverIndicatorRef={hoverIndicatorRef}
        activeIndicatorRef={activeIndicatorRef}
      />

      <ToggleGroup
        type="single"
        orientation="horizontal"
        value={value}
        onValueChange={handleValueChange}
        spacing={0}
        variant="default"
        className="relative z-10 flex h-full w-max min-w-full flex-row flex-nowrap items-stretch justify-start gap-0 bg-transparent p-0 shadow-none"
      >
        {tabs.map((tab) => (
          <TraySegmentItem
            key={tab.id}
            tab={tab}
            setItemRef={setItemRef}
            onHover={syncHoverIndicator}
            onHoverEnd={handleHoverEnd}
            onPointerDown={handlePointerDown}
            onPointerUp={handlePointerUp}
          />
        ))}
      </ToggleGroup>
    </div>
  );
}
