"use client";

import type { RefObject } from "react";

import {
  activeIndicatorClassName,
  indicatorLayerClassName,
  inlineItemClassName,
  pageTabItemClassName,
  usesSegmentActiveIndicator,
  type AppSegmentItem,
  type AppSegmentedControlVariant,
} from "@/components/ui/app-segmented-control-utils";
import { ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

export function SegmentIndicators({
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

export function SegmentToggleItems({
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
