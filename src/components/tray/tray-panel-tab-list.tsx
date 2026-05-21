import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { ScrollFadeRegion } from "@/components/tray/scroll-fade-region";
import {
  TRAY_SEGMENT_ROW_HEIGHT,
  TraySegmentedControl,
} from "@/components/tray/tray-segmented-control";

interface TrayPanelTabListProps {
  tabs: TrayPanelTab[];
  value: string;
  onValueChange: (value: string) => void;
}

export function TrayPanelTabList({ tabs, value, onValueChange }: TrayPanelTabListProps) {
  return (
    <div className="border-border border-b px-3 pb-2">
      <ScrollFadeRegion
        orientation="horizontal"
        rowHeightClassName={TRAY_SEGMENT_ROW_HEIGHT}
        scrollClassName="overscroll-x-contain"
      >
        <TraySegmentedControl tabs={tabs} value={value} onValueChange={onValueChange} />
      </ScrollFadeRegion>
    </div>
  );
}
