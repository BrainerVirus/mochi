import { RefreshCwIcon } from "lucide-react";

import type { ProviderId, UsageSnapshot } from "@/lib/schemas/usage";

import { ProviderIcon } from "@/components/providers/provider-icon";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { formatUpdatedAgo } from "@/lib/utils/format-updated-ago";
import { getProviderLabel } from "@/lib/utils/provider-labels";

import { UsageMeter } from "./usage-meter";

interface ProviderUsageSectionProps {
  snapshot: UsageSnapshot;
  onRefresh?: (provider: ProviderId) => void;
  isRefreshing?: boolean;
  showSeparator?: boolean;
  planLabel?: string | null;
}

export function ProviderUsageSection({
  snapshot,
  onRefresh,
  isRefreshing = false,
  showSeparator = false,
  planLabel = null,
}: ProviderUsageSectionProps) {
  const windows = [snapshot.primary, ...(snapshot.secondary ? [snapshot.secondary] : [])];

  return (
    <section className="flex flex-col gap-2.5">
      {showSeparator ? <Separator className="mb-0.5" /> : null}
      <header className="flex items-start justify-between gap-2">
        <div className="flex min-w-0 flex-col gap-0.5">
          <h3 className="flex items-center gap-1.5 text-sm font-semibold">
            <ProviderIcon provider={snapshot.provider} className="size-4" />
            <span className="truncate">{getProviderLabel(snapshot.provider)}</span>
            {planLabel ? (
              <Badge variant="outline" className="text-[10px] font-normal">
                {planLabel}
              </Badge>
            ) : null}
          </h3>
          <p className="text-muted-foreground text-[11px]">
            {formatUpdatedAgo(snapshot.updated_at)}
          </p>
        </div>
        {onRefresh ? (
          <Button
            variant="ghost"
            size="icon-sm"
            className="shrink-0 cursor-pointer"
            disabled={isRefreshing}
            aria-label={`Refresh ${getProviderLabel(snapshot.provider)} usage`}
            onClick={() => {
              onRefresh(snapshot.provider);
            }}
          >
            <RefreshCwIcon data-icon="inline-start" className={isRefreshing ? "animate-spin" : undefined} />
          </Button>
        ) : null}
      </header>
      <div className="flex flex-col gap-3">
        {windows.map((window) => (
          <UsageMeter
            key={window.label}
            label={window.label}
            usedPercent={window.used_percent}
            remainingPercent={window.remaining_percent}
            resetsAt={window.resets_at}
            compact
          />
        ))}
      </div>
    </section>
  );
}
