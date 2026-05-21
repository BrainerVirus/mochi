import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { ProviderIcon } from "@/components/providers/provider-icon";

/** Matches TabsList row height — keep chevron overlay and tab row in sync. */
export const TRAY_TAB_ROW_HEIGHT = "h-14" as const;
import { ScrollFadeRegion } from "@/components/tray/scroll-fade-region";
import { Progress } from "@/components/ui/progress";
import { TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { getUsageMeterTone, usageMeterToneClasses } from "@/lib/utils/usage-meter-tone";

interface TrayPanelTabListProps {
  tabs: TrayPanelTab[];
}

export function TrayPanelTabList({ tabs }: TrayPanelTabListProps) {
  return (
    <ScrollFadeRegion
      orientation="horizontal"
      rowHeightClassName={TRAY_TAB_ROW_HEIGHT}
      className="border-border border-b"
      scrollClassName="overscroll-x-contain"
    >
      <TabsList
        variant="line"
        className={cn(
          TRAY_TAB_ROW_HEIGHT,
          "w-max min-w-full flex-nowrap items-stretch justify-start gap-0 bg-transparent p-0",
        )}
      >
        {tabs.map((tab) => {
          const tone = getUsageMeterTone(tab.usedPercent);

          return (
            <TabsTrigger
              key={tab.id}
              value={tab.id}
              className={cn(
                "h-full min-w-[4.75rem] max-w-[7rem] shrink-0 flex-none cursor-pointer flex-col items-stretch justify-center gap-1 rounded-none px-2.5 py-2 text-[11px] after:bottom-0",
                "data-active:bg-transparent data-active:shadow-none",
              )}
            >
              <span className="flex w-full min-w-0 items-center gap-1">
                {tab.id !== "overview" ? <ProviderIcon provider={tab.id} /> : null}
                <span className="min-w-0 truncate font-medium">{tab.label}</span>
                {tab.id !== "overview" ? (
                  <span className="text-muted-foreground shrink-0 text-[10px] tabular-nums">
                    {Math.round(tab.remainingPercent)}%
                  </span>
                ) : null}
              </span>
              {tab.id !== "overview" ? (
                <Progress
                  value={tab.usedPercent}
                  className={cn("h-0.5 w-full", usageMeterToneClasses[tone])}
                  aria-hidden
                />
              ) : (
                <span className="text-muted-foreground w-full truncate text-center text-[10px] tabular-nums">
                  {tab.usedPercent > 0
                    ? `${Math.round(tab.remainingPercent)}% left min`
                    : "All clear"}
                </span>
              )}
            </TabsTrigger>
          );
        })}
      </TabsList>
    </ScrollFadeRegion>
  );
}
