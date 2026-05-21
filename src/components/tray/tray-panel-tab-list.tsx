import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { Progress } from "@/components/ui/progress";
import { TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { getUsageMeterTone, usageMeterToneClasses } from "@/lib/utils/usage-meter-tone";

interface TrayPanelTabListProps {
  tabs: TrayPanelTab[];
}

export function TrayPanelTabList({ tabs }: TrayPanelTabListProps) {
  return (
    <TabsList
      variant="line"
      className="border-border h-auto w-full flex-nowrap justify-start gap-0 overflow-x-auto border-b bg-transparent p-0"
    >
      {tabs.map((tab) => {
        const tone = getUsageMeterTone(tab.usedPercent);

        return (
          <TabsTrigger
            key={tab.id}
            value={tab.id}
            className={cn(
              "min-w-0 shrink-0 flex-col gap-1 rounded-none px-2.5 py-2 text-[11px] after:bottom-0",
              "data-active:bg-transparent data-active:shadow-none",
            )}
          >
            <span className="truncate font-medium">{tab.label}</span>
            {tab.id !== "overview" ? (
              <Progress
                value={tab.usedPercent}
                className={cn("h-0.5 w-full", usageMeterToneClasses[tone])}
                aria-hidden
              />
            ) : (
              <span className="text-muted-foreground text-[10px] tabular-nums">
                {tab.usedPercent > 0 ? `${Math.round(tab.usedPercent)}% peak` : "All clear"}
              </span>
            )}
          </TabsTrigger>
        );
      })}
    </TabsList>
  );
}
