import type { TrayPanelTab } from "@/lib/utils/tray-panel-tabs";

import { ProviderIcon } from "@/components/providers/provider-icon";
import { Progress } from "@/components/ui/progress";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";
import { getUsageMeterTone, usageMeterToneClasses } from "@/lib/utils/usage-meter-tone";

/** Fixed row height shared with ScrollFadeRegion chevron overlay math. */
export const TRAY_SEGMENT_ROW_HEIGHT = "h-11" as const;

interface TraySegmentedControlProps {
  tabs: TrayPanelTab[];
  value: string;
  onValueChange: (value: string) => void;
}

export function TraySegmentedControl({ tabs, value, onValueChange }: TraySegmentedControlProps) {
  return (
    <ToggleGroup
      type="single"
      value={value}
      onValueChange={(next) => {
        if (next) {
          onValueChange(next);
        }
      }}
      spacing={0}
      variant="outline"
      className={cn(
        TRAY_SEGMENT_ROW_HEIGHT,
        "w-max min-w-full flex-nowrap items-stretch justify-start gap-0 rounded-lg bg-muted/50 p-0.5",
      )}
    >
      {tabs.map((tab) => {
        const tone = getUsageMeterTone(tab.usedPercent);
        const isOverview = tab.id === "overview";

        return (
          <ToggleGroupItem
            key={tab.id}
            value={tab.id}
            aria-label={tab.label}
            className={cn(
              "h-full min-w-[4.75rem] max-w-[7rem] shrink-0 flex-none cursor-pointer flex-col items-stretch justify-center gap-0.5 rounded-md border-0 px-2 py-1 text-[11px] shadow-none",
              "bg-transparent text-muted-foreground hover:bg-transparent hover:text-foreground",
              "data-[state=on]:bg-background data-[state=on]:text-foreground data-[state=on]:shadow-sm",
            )}
          >
            <span className="flex w-full min-w-0 items-center gap-1">
              {tab.id !== "overview" ? <ProviderIcon provider={tab.id} /> : null}
              <span className="min-w-0 truncate font-medium">{tab.label}</span>
              {!isOverview ? (
                <span className="shrink-0 text-[10px] tabular-nums opacity-80">
                  {Math.round(tab.remainingPercent)}%
                </span>
              ) : null}
            </span>
            {isOverview ? (
              <span className="w-full truncate text-center text-[10px] tabular-nums opacity-80">
                {tab.usedPercent > 0
                  ? `${Math.round(tab.remainingPercent)}% left min`
                  : "All clear"}
              </span>
            ) : (
              <Progress
                value={tab.usedPercent}
                className={cn("h-0.5 w-full", usageMeterToneClasses[tone])}
                aria-hidden
              />
            )}
          </ToggleGroupItem>
        );
      })}
    </ToggleGroup>
  );
}
