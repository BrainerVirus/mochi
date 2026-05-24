"use client";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { LayoutGridIcon } from "lucide-react";
import { ProviderIcon } from "@/components/providers/provider-icon";
import {
  SETTINGS_PAGE_TAB_DEFAULTS,
  TRAY_PAGE_TAB_DEFAULTS,
} from "@/components/tray/tray-segmented-control-config";
import { AppSegmentedControl } from "@/components/ui/app-segmented-control";
import type { AppSegmentItem } from "@/components/ui/app-segmented-control-utils";

interface PageTabSegmentedControlProps {
  items: AppSegmentItem[];
  value: string;
  onValueChange: (value: string) => void;
  className?: string;
  contentReady?: boolean;
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
  contentReady = true,
}: PageTabSegmentedControlProps) {
  return (
    <AppSegmentedControl
      items={items}
      value={value}
      onValueChange={onValueChange}
      className={className}
      contentReady={contentReady}
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
