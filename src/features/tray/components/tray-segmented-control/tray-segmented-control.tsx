"use client";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { LayoutGridIcon } from "lucide-react";
import { ProviderIcon } from "@/components/providers/provider-icon";
import {
  AppSegmentedControl,
  AppSegmentedControlView,
} from "@/components/ui/app-segmented-control";
import type { AppSegmentItem } from "@/components/ui/app-segmented-control-utils";
import type { useAppSegmentControlState } from "@/components/ui/use-app-segment-control-state";
import {
  SETTINGS_PAGE_TAB_DEFAULTS,
  TRAY_PAGE_TAB_DEFAULTS,
} from "@/features/tray/components/tray-segmented-control-config";

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

function traySegmentItems(tabs: TrayPanelTab[]): AppSegmentItem[] {
  return tabs.map((tab) => ({
    id: tab.id,
    label: tab.label,
    icon:
      tab.id === "overview" ? <LayoutGridIcon aria-hidden /> : <ProviderIcon provider={tab.id} />,
  }));
}

export function TraySegmentedControl({ tabs, value, onValueChange }: TraySegmentedControlProps) {
  return (
    <PageTabSegmentedControl
      items={traySegmentItems(tabs)}
      value={value}
      onValueChange={onValueChange}
    />
  );
}

export function TraySegmentedControlView({
  tabs,
  value,
  state,
}: Omit<TraySegmentedControlProps, "onValueChange"> & {
  state: ReturnType<typeof useAppSegmentControlState>;
}) {
  return (
    <AppSegmentedControlView
      items={traySegmentItems(tabs)}
      value={value}
      state={state}
      {...TRAY_PAGE_TAB_DEFAULTS}
    />
  );
}
