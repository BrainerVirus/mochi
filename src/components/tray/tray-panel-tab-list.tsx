import { useCallback } from "react";

import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { ScrollFadeRegion } from "@/components/tray/scroll-fade-region";
import { cycleTrayPanelTabs } from "@/components/tray/tray-panel-tab-cycle";
import { TraySegmentedControl } from "@/components/tray/tray-segmented-control";
import { TRAY_SEGMENT_ROW_HEIGHT } from "@/components/tray/tray-segmented-control-config";

const TAB_FADE_INSET = 40;

interface TrayPanelTabListProps {
  tabs: TrayPanelTab[];
  value: string;
  onValueChange: (value: string) => void;
}

export function TrayPanelTabList({ tabs, value, onValueChange }: TrayPanelTabListProps) {
  const handleCycleForward = useCallback(
    (scrollEl: HTMLDivElement) => {
      cycleTrayPanelTabs(scrollEl, tabs, value, "forward", onValueChange, TAB_FADE_INSET);
    },
    [onValueChange, tabs, value],
  );

  const handleCycleBackward = useCallback(
    (scrollEl: HTMLDivElement) => {
      cycleTrayPanelTabs(scrollEl, tabs, value, "backward", onValueChange, TAB_FADE_INSET);
    },
    [onValueChange, tabs, value],
  );

  return (
    <div
      data-tray-tab-strip
      className="border-border min-w-0 overflow-hidden rounded-t-[var(--radius-tray-panel)] border-b px-3 pb-2"
    >
      <ScrollFadeRegion
        orientation="horizontal"
        className="w-full min-w-0 overflow-hidden"
        rowHeightClassName={TRAY_SEGMENT_ROW_HEIGHT}
        scrollClassName="overscroll-x-contain"
        fadeInset={TAB_FADE_INSET}
        onCycleForward={handleCycleForward}
        onCycleBackward={handleCycleBackward}
      >
        <TraySegmentedControl tabs={tabs} value={value} onValueChange={onValueChange} />
      </ScrollFadeRegion>
    </div>
  );
}
