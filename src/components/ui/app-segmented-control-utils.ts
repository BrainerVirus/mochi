import type { ReactNode } from "react";

import { cn } from "@/lib/utils/cn";

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

export function indicatorLayerClassName(indicatorRadiusClass: string): string {
  return cn(
    "pointer-events-none absolute inset-y-0.5 left-0 will-change-[transform,width]",
    indicatorRadiusClass,
  );
}

export const pageTabItemClassName = cn(
  "relative z-10 inline-flex h-full min-w-[4.5rem] shrink-0 flex-none cursor-pointer flex-row items-center justify-center gap-1.5 rounded-none border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:bg-transparent hover:text-foreground",
  "data-[state=on]:bg-transparent data-[state=on]:font-semibold data-[state=on]:text-foreground data-[state=on]:shadow-none",
  "first:rounded-none last:rounded-none",
);

export const inlineItemClassName = cn(
  "relative z-10 inline-flex h-full min-w-0 flex-1 cursor-pointer flex-row items-center justify-center gap-1.5 rounded-md border-0 px-3 shadow-none",
  "bg-transparent text-muted-foreground hover:text-foreground",
  "data-[state=on]:bg-transparent data-[state=on]:font-medium data-[state=on]:text-primary-foreground data-[state=on]:shadow-none",
);

export function activeIndicatorClassName(variant: AppSegmentedControlVariant): string {
  if (variant === "page-tabs") {
    return "bg-[var(--app-segment-active)] shadow-sm ring-1 ring-[var(--app-segment-stroke)]";
  }

  return "bg-primary";
}
