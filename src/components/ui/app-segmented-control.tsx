"use client";

import { useCallback, useRef, type ReactNode, type RefObject } from "react";

import {
  useTraySegmentIndicators,
  type UseTraySegmentIndicatorsOptions,
} from "@/components/tray/use-tray-segment-indicators";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

export interface AppSegmentItem {
  id: string;
  label: string;
  icon?: ReactNode;
}

export type AppSegmentedControlVariant = "page-tabs" | "inline";

export type AppSegmentedControlLayout = "tray" | "settings";

export function usesPageTabIndicators(variant: AppSegmentedControlVariant): boolean {
  return variant === "page-tabs";
}

export function usesSegmentActiveIndicator(variant: AppSegmentedControlVariant): boolean {
  return variant === "page-tabs" || variant === "inline";
}

export function usesSegmentHoverIndicator(
  variant: AppSegmentedControlVariant,
  layout: AppSegmentedControlLayout = "tray",
): boolean {
  return variant === "page-tabs" && layout === "tray";
}

/** Fixed radius tokens — tray page tabs only; not used in settings (.app-window --radius). */
export const APP_SEGMENT_INDICATOR_RADIUS_CLASS = "rounded-app-segment-indicator" as const;
export const APP_SEGMENT_TRACK_RADIUS_CLASS = "rounded-app-segment-track" as const;

/** Settings page tabs follow .app-window --radius via Tailwind rounded-* utilities. */
export const SETTINGS_SEGMENT_INDICATOR_RADIUS_CLASS = "rounded-md" as const;
export const SETTINGS_SEGMENT_TRACK_RADIUS_CLASS = "rounded-lg" as const;

export const INLINE_SEGMENT_INDICATOR_RADIUS_CLASS = "rounded-md" as const;

export function resolvePageTabRadiusClasses(layout: AppSegmentedControlLayout): {
  track: string;
  indicator: string;
} {
  if (layout === "settings") {
    return {
      track: SETTINGS_SEGMENT_TRACK_RADIUS_CLASS,
      indicator: SETTINGS_SEGMENT_INDICATOR_RADIUS_CLASS,
    };
  }

  return {
    track: APP_SEGMENT_TRACK_RADIUS_CLASS,
    indicator: APP_SEGMENT_INDICATOR_RADIUS_CLASS,
  };
}

function indicatorLayerClassName(indicatorRadiusClass: string): string {
  return cn(
    "pointer-events-none absolute inset-y-0.5 left-0 will-change-[transform,width]",
    indicatorRadiusClass,
  );
}

const pageTabItemClassName = cn(
  "relative z-10 inline-flex h-full min-w-[4.5rem] shrink-0 flex-none cursor-pointer flex-row items-center justify-center gap-1.5 rounded-none border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:bg-transparent hover:text-foreground",
  "data-[state=on]:bg-transparent data-[state=on]:font-semibold data-[state=on]:text-foreground data-[state=on]:shadow-none",
  "first:rounded-none last:rounded-none",
);

const inlineItemClassName = cn(
  "relative z-10 inline-flex h-full min-w-0 flex-1 cursor-pointer flex-row items-center justify-center gap-1.5 rounded-md border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:text-foreground",
  "data-[state=on]:bg-transparent data-[state=on]:font-medium data-[state=on]:text-primary-foreground data-[state=on]:shadow-none",
);

function activeIndicatorClassName(variant: AppSegmentedControlVariant): string {
  if (variant === "page-tabs") {
    return "bg-[var(--app-segment-active)] shadow-sm ring-1 ring-[var(--app-segment-stroke)]";
  }

  return "bg-primary";
}

function SegmentIndicators({
  hoverIndicatorRef,
  activeIndicatorRef,
  indicatorRadiusClass,
  variant,
  showHover,
}: {
  hoverIndicatorRef: RefObject<HTMLDivElement | null>;
  activeIndicatorRef: RefObject<HTMLDivElement | null>;
  indicatorRadiusClass: string;
  variant: AppSegmentedControlVariant;
  showHover: boolean;
}) {
  const layerClassName = indicatorLayerClassName(indicatorRadiusClass);

  return (
    <>
      {showHover ? (
        <div
          ref={hoverIndicatorRef}
          data-segment-hover-indicator
          aria-hidden
          className={cn(layerClassName, "z-[1] invisible bg-[var(--app-segment-hover)]")}
          style={{ width: 0 }}
        />
      ) : null}
      <div
        ref={activeIndicatorRef}
        data-segment-indicator
        aria-hidden
        className={cn(layerClassName, "z-[2] invisible", activeIndicatorClassName(variant))}
        style={{ width: 0 }}
      />
    </>
  );
}

function useAppSegmentControlState(
  value: string,
  itemCount: number,
  onValueChange: (value: string) => void,
  indicatorOptions: UseTraySegmentIndicatorsOptions & {
    enabled: boolean;
    contentReady?: boolean;
  },
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

  const { syncHoverIndicator, handleRailLeave, handleSegmentValueChange, pillReady } =
    useTraySegmentIndicators(
      trackRef,
      activeIndicatorRef,
      hoverIndicatorRef,
      value,
      indicatorOptions.enabled ? itemCount : 0,
      itemRefs,
      {
        showHover: indicatorOptions.showHover,
        contentReady: indicatorOptions.contentReady,
      },
    );

  const handleValueChange = useCallback(
    (next: string) => {
      if (next) {
        if (indicatorOptions.enabled) {
          handleSegmentValueChange(next, onValueChange);
          return;
        }
        onValueChange(next);
      }
    },
    [handleSegmentValueChange, indicatorOptions.enabled, onValueChange],
  );

  return {
    trackRef,
    activeIndicatorRef,
    hoverIndicatorRef,
    setItemRef,
    syncHoverIndicator,
    handleRailLeave,
    handleValueChange,
    pillReady,
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
  /** Page-tabs only: tray keeps pill radii + scroll; settings uses full width + --radius rounding. */
  layout?: AppSegmentedControlLayout;
  /** When false, tab pill snaps until content is ready (e.g. settings query loading). */
  contentReady?: boolean;
}

function SegmentToggleItems({
  items,
  variant,
  stretchItems,
  setItemRef,
  syncHoverIndicator,
  showHover,
}: {
  items: AppSegmentItem[];
  variant: AppSegmentedControlVariant;
  stretchItems: boolean;
  setItemRef: (id: string, element: HTMLButtonElement | null) => void;
  syncHoverIndicator: (id: string) => void;
  showHover: boolean;
}) {
  const isPageTabs = variant === "page-tabs";
  const usesIndicators = usesSegmentActiveIndicator(variant);

  return items.map((item) => (
    <ToggleGroupItem
      key={item.id}
      ref={
        usesIndicators
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
      onPointerEnter={showHover ? () => syncHoverIndicator(item.id) : undefined}
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
  layout = "tray",
  contentReady = true,
}: AppSegmentedControlProps) {
  const isPageTabs = usesPageTabIndicators(variant);
  const showHover = usesSegmentHoverIndicator(variant);
  const usesIndicators = usesSegmentActiveIndicator(variant);
  const pageTabRadius = resolvePageTabRadiusClasses(layout);
  const indicatorRadiusClass = isPageTabs
    ? pageTabRadius.indicator
    : INLINE_SEGMENT_INDICATOR_RADIUS_CLASS;
  const state = useAppSegmentControlState(value, items.length, onValueChange, {
    enabled: usesIndicators,
    showHover,
    contentReady,
  });
  const trackClassName = isPageTabs
    ? "bg-[var(--app-segment-track)]"
    : "bg-[var(--app-segment-inline-track)]";
  const blockInteractionUntilPlaced = usesIndicators && layout === "settings" && !state.pillReady;

  return (
    <div
      ref={usesIndicators ? state.trackRef : undefined}
      onPointerLeave={showHover ? state.handleRailLeave : undefined}
      data-segment-variant={variant}
      data-segment-layout={isPageTabs ? layout : undefined}
      className={cn(
        rowHeight,
        "relative isolate p-0.5",
        isPageTabs ? pageTabRadius.track : "rounded-lg",
        trackClassName,
        stretchItems ? "w-full" : "w-max min-w-full",
        blockInteractionUntilPlaced && "pointer-events-none",
        className,
      )}
    >
      {usesIndicators ? (
        <SegmentIndicators
          hoverIndicatorRef={state.hoverIndicatorRef}
          activeIndicatorRef={state.activeIndicatorRef}
          indicatorRadiusClass={indicatorRadiusClass}
          variant={variant}
          showHover={showHover}
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
          variant={variant}
          stretchItems={stretchItems}
          setItemRef={state.setItemRef}
          syncHoverIndicator={state.syncHoverIndicator}
          showHover={showHover}
        />
      </ToggleGroup>
    </div>
  );
}
