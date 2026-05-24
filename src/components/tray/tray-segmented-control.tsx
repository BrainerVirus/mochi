"use client";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { LayoutGridIcon } from "lucide-react";
import { ProviderIcon } from "@/components/providers/provider-icon";
import { AppSegmentedControl } from "@/components/ui/app-segmented-control";

/** Fixed row height shared with ScrollFadeRegion chevron overlay math. */
export const TRAY_SEGMENT_ROW_HEIGHT = "h-11" as const;

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

  return (
    <AppSegmentedControl
      items={items}
      value={value}
      onValueChange={onValueChange}
      variant="page-tabs"
      rowHeight={TRAY_SEGMENT_ROW_HEIGHT}
      stretchItems={false}
    />
  );
}
