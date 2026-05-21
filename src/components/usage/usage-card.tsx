import type { UsageSnapshot } from "@/lib/schemas/usage";

import { ProviderIcon } from "@/components/providers/provider-icon";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { getProviderLabel } from "@/lib/utils/provider-labels";

import { UsageMeter } from "./usage-meter";

interface UsageCardProps {
  snapshot: UsageSnapshot;
  compact?: boolean;
  showHeader?: boolean;
}

export function UsageCard({ snapshot, compact = false, showHeader = true }: UsageCardProps) {
  return (
    <section className="flex flex-col gap-2.5">
      {showHeader ? (
        <header className="flex items-center justify-between gap-2">
          <h3 className="flex items-center gap-1.5 text-sm font-medium">
            <ProviderIcon provider={snapshot.provider} />
            {getProviderLabel(snapshot.provider)}
          </h3>
          <Badge variant="outline" className="text-[10px]">
            {snapshot.source}
          </Badge>
        </header>
      ) : null}
      <UsageMeter
        label={snapshot.primary.label}
        usedPercent={snapshot.primary.used_percent}
        remainingPercent={snapshot.primary.remaining_percent}
        resetsAt={snapshot.primary.resets_at}
        compact={compact}
      />
      {snapshot.secondary ? (
        <>
          <Separator />
          <UsageMeter
            label={snapshot.secondary.label}
            usedPercent={snapshot.secondary.used_percent}
            remainingPercent={snapshot.secondary.remaining_percent}
            resetsAt={snapshot.secondary.resets_at}
            compact={compact}
          />
        </>
      ) : null}
    </section>
  );
}
