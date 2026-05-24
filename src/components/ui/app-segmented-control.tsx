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

export type AppSegmentedControlVariant = "page-tabs" | "inline";

export function usesPageTabIndicators(variant: AppSegmentedControlVariant): boolean {
  return variant === "page-tabs";
}

const indicatorLayerClassName =
  "pointer-events-none absolute inset-y-0.5 left-0 rounded-md will-change-[transform,width]";

const pageTabItemClassName = cn(
  "relative z-10 inline-flex h-full min-w-[4.5rem] shrink-0 flex-none cursor-pointer flex-row items-center justify-center gap-1.5 rounded-none border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:bg-transparent hover:text-foreground",
  "data-[state=on]:bg-transparent data-[state=on]:font-semibold data-[state=on]:text-foreground data-[state=on]:shadow-none",
  "first:rounded-none last:rounded-none",
);

const inlineItemClassName = cn(
  "inline-flex h-full min-w-0 flex-1 cursor-pointer flex-row items-center justify-center gap-1.5 rounded-md border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:text-foreground",
  "data-[state=on]:bg-primary data-[state=on]:font-medium data-[state=on]:text-primary-foreground data-[state=on]:shadow-none",
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
        className={cn(indicatorLayerClassName, "z-[2] invisible bg-[var(--app-segment-active)]")}
        style={{ width: 0 }}
      />
    </>
  );
}

function useAppSegmentControlState(
  value: string,
  itemCount: number,
  onValueChange: (value: string) => void,
  enabled: boolean,
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
      enabled ? itemCount : 0,
      itemRefs,
    );

  const handleValueChange = useCallback(
    (next: string) => {
      if (next) {
        if (enabled) {
          handleSegmentValueChange(next, onValueChange);
          return;
        }
        onValueChange(next);
      }
    },
    [enabled, handleSegmentValueChange, onValueChange],
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
  stretchItems?: boolean;
  variant?: AppSegmentedControlVariant;
}

function SegmentToggleItems({
  items,
  isPageTabs,
  stretchItems,
  setItemRef,
  syncHoverIndicator,
}: {
  items: AppSegmentItem[];
  isPageTabs: boolean;
  stretchItems: boolean;
  setItemRef: (id: string, element: HTMLButtonElement | null) => void;
  syncHoverIndicator: (id: string) => void;
}) {
  return items.map((item) => (
    <ToggleGroupItem
      key={item.id}
      ref={
        isPageTabs
          ? (element) => {
              setItemRef(item.id, element);
            }
          : undefined
      }
      data-tray-tab-id={isPageTabs ? item.id : undefined}
      value={item.id}
      aria-label={item.label}
      className={cn(
        isPageTabs ? pageTabItemClassName : inlineItemClassName,
        stretchItems ? "min-w-0 flex-1" : "min-w-[4.75rem] max-w-[7.5rem] shrink-0 flex-none",
      )}
      onPointerEnter={isPageTabs ? () => syncHoverIndicator(item.id) : undefined}
    >
      {item.icon ? (
        <span className="flex size-4 shrink-0 items-center justify-center opacity-90">
          {item.icon}
        </span>
      ) : null}
      <span className="min-w-0 truncate text-xs font-medium">{item.label}</span>
    </ToggleGroupItem>
  ));
}

export function AppSegmentedControl({
  items,
  value,
  onValueChange,
  className,
  rowHeight = "h-9",
  stretchItems = true,
  variant = "page-tabs",
}: AppSegmentedControlProps) {
  const isPageTabs = usesPageTabIndicators(variant);
  const state = useAppSegmentControlState(value, items.length, onValueChange, isPageTabs);
  const trackClassName = isPageTabs
    ? "bg-[var(--app-segment-track)]"
    : "bg-[var(--app-segment-inline-track)]";

  return (
    <div
      ref={isPageTabs ? state.trackRef : undefined}
      onPointerLeave={isPageTabs ? state.handleRailLeave : undefined}
      data-segment-variant={variant}
      className={cn(
        rowHeight,
        "relative isolate rounded-lg p-0.5",
        trackClassName,
        stretchItems ? "w-full" : "w-max min-w-full",
        className,
      )}
    >
      {isPageTabs ? (
        <SegmentIndicators
          hoverIndicatorRef={state.hoverIndicatorRef}
          activeIndicatorRef={state.activeIndicatorRef}
        />
      ) : null}

      <ToggleGroup
        type="single"
        orientation="horizontal"
        value={value}
        onValueChange={state.handleValueChange}
        spacing={0}
        variant="default"
        className="relative z-10 flex h-full w-full flex-row flex-nowrap items-stretch justify-stretch gap-0 bg-transparent p-0 shadow-none"
      >
        <SegmentToggleItems
          items={items}
          isPageTabs={isPageTabs}
          stretchItems={stretchItems}
          setItemRef={state.setItemRef}
          syncHoverIndicator={state.syncHoverIndicator}
        />
      </ToggleGroup>
    </div>
  );
}
