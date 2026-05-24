"use client";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { LayoutGridIcon } from "lucide-react";
import { ProviderIcon } from "@/components/providers/provider-icon";
import { AppSegmentedControl, type AppSegmentItem } from "@/components/ui/app-segmented-control";

/** Fixed row height shared with ScrollFadeRegion chevron overlay math. */
export const TRAY_SEGMENT_ROW_HEIGHT = "h-11" as const;

/** Tray page-tab strip — scrollable segments with fixed 1.5rem pill radii. */
export const TRAY_PAGE_TAB_DEFAULTS = {
  variant: "page-tabs" as const,
  rowHeight: TRAY_SEGMENT_ROW_HEIGHT,
  stretchItems: false,
  layout: "tray" as const,
};

/** Settings page-tab strip — full-width equal segments with .app-window --radius rounding. */
export const SETTINGS_PAGE_TAB_DEFAULTS = {
  variant: "page-tabs" as const,
  rowHeight: "h-9" as const,
  stretchItems: true,
  layout: "settings" as const,
};

interface PageTabSegmentedControlProps {
  items: AppSegmentItem[];
  value: string;
  onValueChange: (value: string) => void;
  className?: string;
}

/** Tray page-level tab strip (providers overview, per-provider tabs). */
export function PageTabSegmentedControl({
  items,
  value,
  onValueChange,
  className,
}: PageTabSegmentedControlProps) {
  return (
    <AppSegmentedControl
      items={items}
      value={value}
      onValueChange={onValueChange}
      className={className}
      {...TRAY_PAGE_TAB_DEFAULTS}
    />
  );
}

/** Settings General/Providers tab strip — full width, moderate radius. */
export function SettingsTabSegmentedControl({
  items,
  value,
  onValueChange,
  className,
}: PageTabSegmentedControlProps) {
  return (
    <AppSegmentedControl
      items={items}
      value={value}
      onValueChange={onValueChange}
      className={className}
      {...SETTINGS_PAGE_TAB_DEFAULTS}
    />
  );
}

interface TraySegmentedControlProps {
  tabs: TrayPanelTab[];
  value: string;
  onValueChange: (value: string) => void;
}

export function TraySegmentedControl({ tabs, value, onValueChange }: TraySegmentedControlProps) {
  const items = tabs.map((tab) => ({
    id: tab.id,
    label: tab.label,
    icon:
      tab.id === "overview" ? <LayoutGridIcon aria-hidden /> : <ProviderIcon provider={tab.id} />,
  }));

  return <PageTabSegmentedControl items={items} value={value} onValueChange={onValueChange} />;
}
