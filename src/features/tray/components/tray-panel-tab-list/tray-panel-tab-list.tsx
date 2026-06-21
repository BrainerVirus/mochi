import { useCallback } from "react";

import { useAppSegmentControlState } from "@/components/ui/use-app-segment-control-state";
import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { ScrollFadeRegion } from "@/features/tray/components/scroll-fade-region";
import { cycleTrayPanelTabs } from "@/features/tray/components/tray-panel-tab-cycle";
import { TraySegmentedControlView } from "@/features/tray/components/tray-segmented-control";
import { TRAY_SEGMENT_ROW_HEIGHT } from "@/features/tray/components/tray-segmented-control-config";

const TAB_FADE_INSET = 40;

interface TrayPanelTabListProps {
  tabs: TrayPanelTab[];
  value: string;
  onValueChange: (value: string) => void;
}

export function TrayPanelTabList({ tabs, value, onValueChange }: TrayPanelTabListProps) {
  const segmentState = useAppSegmentControlState(value, tabs.length, onValueChange, {
    enabled: true,
    showHover: true,
    contentReady: true,
  });
  const handleValueChange = segmentState.handleValueChange;
  const handleCycleForward = useCallback(
    (scrollEl: HTMLDivElement) => {
      cycleTrayPanelTabs(scrollEl, tabs, value, "forward", handleValueChange, TAB_FADE_INSET);
    },
    [handleValueChange, tabs, value],
  );

  const handleCycleBackward = useCallback(
    (scrollEl: HTMLDivElement) => {
      cycleTrayPanelTabs(scrollEl, tabs, value, "backward", handleValueChange, TAB_FADE_INSET);
    },
    [handleValueChange, tabs, value],
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
        <TraySegmentedControlView tabs={tabs} value={value} state={segmentState} />
      </ScrollFadeRegion>
    </div>
  );
}
