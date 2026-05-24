"use client";

import { useCallback, useRef, type ReactNode, type RefObject } from "react";

import { useTraySegmentIndicators } from "@/components/tray/use-tray-segment-indicators";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

export interface AppSegmentItem {
  id: string;
  label: string;
  icon?: ReactNode;
}

const indicatorLayerClassName =
  "pointer-events-none absolute inset-y-0.5 left-0 rounded-md will-change-[transform,width]";

const segmentItemClassName = cn(
  "relative z-10 inline-flex h-full min-w-[4.5rem] shrink-0 flex-none cursor-pointer flex-row items-center justify-center gap-1.5 rounded-none border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:bg-transparent hover:text-foreground",
  "data-[state=on]:bg-transparent data-[state=on]:font-semibold data-[state=on]:text-foreground data-[state=on]:shadow-none",
  "first:rounded-none last:rounded-none",
);

function SegmentIndicators({
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
        className={cn(indicatorLayerClassName, "z-[1] invisible bg-[var(--app-segment-hover)]")}
        style={{ width: 0 }}
      />
      <div
        ref={activeIndicatorRef}
        data-segment-indicator
        aria-hidden
        className={cn(
          indicatorLayerClassName,
          "z-[2] invisible bg-[var(--app-segment-active)] shadow-sm ring-1 ring-[var(--app-segment-stroke)]",
        )}
        style={{ width: 0 }}
      />
    </>
  );
}

function useAppSegmentControlState(
  value: string,
  itemCount: number,
  onValueChange: (value: string) => void,
) {
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

  const { syncHoverIndicator, handleRailLeave, handleSegmentValueChange } =
    useTraySegmentIndicators(
      trackRef,
      activeIndicatorRef,
      hoverIndicatorRef,
      value,
      itemCount,
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

  return {
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    setItemRef,
    syncHoverIndicator,
    handleRailLeave,
    handleValueChange,
  };
}

interface AppSegmentedControlProps {
  items: AppSegmentItem[];
  value: string;
  onValueChange: (value: string) => void;
  className?: string;
  rowHeight?: string;
  /** When true, segments share width equally; tray uses false for scrollable tabs. */
  stretchItems?: boolean;
}

export function AppSegmentedControl({
  items,
  value,
  onValueChange,
  className,
  rowHeight = "h-9",
  stretchItems = true,
}: AppSegmentedControlProps) {
  const {
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    setItemRef,
    syncHoverIndicator,
    handleRailLeave,
    handleValueChange,
  } = useAppSegmentControlState(value, items.length, onValueChange);

  return (
    <div
      ref={trackRef}
      onPointerLeave={handleRailLeave}
      className={cn(
        rowHeight,
        "relative isolate rounded-lg bg-[var(--app-segment-track)] p-0.5",
        stretchItems ? "w-full" : "w-max min-w-full",
        className,
      )}
    >
      <SegmentIndicators
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
        className="relative z-10 flex h-full w-full flex-row flex-nowrap items-stretch justify-stretch gap-0 bg-transparent p-0 shadow-none"
      >
        {items.map((item) => (
          <ToggleGroupItem
            key={item.id}
            ref={(element) => {
              setItemRef(item.id, element);
            }}
            data-tray-tab-id={item.id}
            value={item.id}
            aria-label={item.label}
            className={cn(
              segmentItemClassName,
              stretchItems ? "min-w-0 flex-1" : "min-w-[4.75rem] max-w-[7.5rem] shrink-0 flex-none",
            )}
            onPointerEnter={() => syncHoverIndicator(item.id)}
          >
            {item.icon ? (
              <span className="flex size-4 shrink-0 items-center justify-center opacity-90">
                {item.icon}
              </span>
            ) : null}
            <span className="min-w-0 truncate text-xs font-medium">{item.label}</span>
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}
