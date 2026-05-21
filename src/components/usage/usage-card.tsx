import type { UsageSnapshot } from "@/lib/schemas/usage";

import { UsageMeter } from "./usage-meter";

interface UsageCardProps {
  snapshot: UsageSnapshot;
}

export function UsageCard({ snapshot }: UsageCardProps) {
  return (
    <article className="rounded-mochi bg-white p-4 shadow-sm ring-1 ring-slate-200">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold capitalize">{snapshot.provider}</h2>
        <span className="bg-mochi-matcha/40 rounded-full px-2 py-1 text-xs text-slate-700">
          {snapshot.source}
        </span>
      </div>
      <div className="mt-4 flex flex-col gap-3">
        <UsageMeter label={snapshot.primary.label} usedPercent={snapshot.primary.used_percent} />
        {snapshot.secondary ? (
          <UsageMeter
            label={snapshot.secondary.label}
            usedPercent={snapshot.secondary.used_percent}
          />
        ) : null}
      </div>
    </article>
  );
}
