"use client";

import { ToggleGroup } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils/cn";

import {
  SegmentIndicators,
  SegmentToggleItems,
} from "@/components/ui/app-segmented-control-segments";
import {
  INLINE_SEGMENT_INDICATOR_RADIUS_CLASS,
  resolvePageTabRadiusClasses,
  usesPageTabIndicators,
  usesSegmentActiveIndicator,
  usesSegmentHoverIndicator,
  type AppSegmentItem,
  type AppSegmentedControlLayout,
  type AppSegmentedControlVariant,
} from "@/components/ui/app-segmented-control-utils";
import { useAppSegmentControlState } from "@/components/ui/use-app-segment-control-state";

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
