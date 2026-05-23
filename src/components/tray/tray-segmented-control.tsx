"use client";

import { useCallback, useRef } from "react";

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

  const { syncHoverIndicator } = useTraySegmentIndicators(
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    value,
    tabs.length,
    itemRefs,
  );

  return (
    <div
      ref={trackRef}
      className={cn(
        TRAY_SEGMENT_ROW_HEIGHT,
        "relative isolate w-max min-w-full rounded-lg bg-muted/40 p-0.5",
      )}
    >
      <div
        ref={hoverIndicatorRef}
        data-segment-hover-indicator
        aria-hidden
        className={cn(indicatorLayerClassName, "z-[1] bg-background/25 invisible")}
        style={{ width: 0 }}
      />
      <div
        ref={activeIndicatorRef}
        data-segment-indicator
        aria-hidden
        className={cn(indicatorLayerClassName, "z-[2] bg-background invisible shadow-sm")}
        style={{ width: 0 }}
      />

      <ToggleGroup
        type="single"
        orientation="horizontal"
        value={value}
        onValueChange={(next) => next && onValueChange(next)}
        spacing={0}
        variant="default"
        className="relative z-10 flex h-full w-max min-w-full flex-row flex-nowrap items-stretch justify-start gap-0 bg-transparent p-0 shadow-none"
      >
        {tabs.map((tab) => (
          <TraySegmentItem
            key={tab.id}
            tab={tab}
            setItemRef={setItemRef}
            onHover={(id) => syncHoverIndicator(id, true)}
            onHoverEnd={() => syncHoverIndicator(null, true)}
          />
        ))}
      </ToggleGroup>
    </div>
  );
}
