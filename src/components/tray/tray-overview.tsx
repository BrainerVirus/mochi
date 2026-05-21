import type { UsageSnapshot } from "@/lib/schemas/usage";
import type { OverviewMetrics } from "@/lib/utils/tray-panel-tabs";

import { Separator } from "@/components/ui/separator";
import { UsageMeter } from "@/components/usage/usage-meter";
import { cn } from "@/lib/utils";
import { getProviderLabel } from "@/lib/utils/provider-labels";
import { getUsageMeterTone, usageMeterFillClasses } from "@/lib/utils/usage-meter-tone";

interface TrayOverviewProps {
  snapshots: UsageSnapshot[];
  metrics: OverviewMetrics;
}

export function TrayOverview({ snapshots, metrics }: TrayOverviewProps) {
  return (
    <div className="flex flex-col gap-3">
      <dl className="grid grid-cols-2 gap-2">
        <MetricTile label="Providers" value={String(metrics.providerCount)} />
        <MetricTile label="Highest" value={`${metrics.highestUsedPercent}%`} />
        <MetricTile label="Average" value={`${metrics.averageUsedPercent}%`} />
        <MetricTile label="Healthy" value={String(metrics.healthyCount)} />
      </dl>

      <Separator />

      <div className="flex flex-col gap-1.5">
        <p className="text-muted-foreground text-[11px] font-medium tracking-wide uppercase">
          Usage by provider
        </p>
        <div className="flex h-14 items-end gap-1.5">
          {snapshots.map((snapshot) => {
            const used = Math.max(0, Math.min(100, snapshot.primary.used_percent));
            const tone = getUsageMeterTone(used);

            return (
              <div
                key={snapshot.provider}
                className="flex min-w-0 flex-1 flex-col items-center gap-1"
              >
                <div className="bg-muted flex h-10 w-full items-end overflow-hidden rounded-sm">
                  <div
                    className={cn("w-full rounded-sm transition-all", usageMeterFillClasses[tone])}
                    style={{ height: `${Math.max(used, 4)}%` }}
                    title={`${getProviderLabel(snapshot.provider)}: ${Math.round(used)}% used`}
                  />
                </div>
                <span className="text-muted-foreground w-full truncate text-center text-[10px]">
                  {getProviderLabel(snapshot.provider)}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      <Separator />

      <ul className="flex flex-col gap-3">
        {snapshots.map((snapshot) => (
          <li key={snapshot.provider}>
            <UsageMeter
              label={getProviderLabel(snapshot.provider)}
              usedPercent={snapshot.primary.used_percent}
              remainingPercent={snapshot.primary.remaining_percent}
              resetsAt={snapshot.primary.resets_at}
              compact
            />
          </li>
        ))}
      </ul>
    </div>
  );
}

function MetricTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-muted/50 flex flex-col gap-0.5 rounded-md px-2.5 py-2">
      <dt className="text-muted-foreground text-[10px] tracking-wide uppercase">{label}</dt>
      <dd className="text-sm font-semibold tabular-nums">{value}</dd>
    </div>
  );
}
