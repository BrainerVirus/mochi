import type { UsageSnapshot } from "@/lib/schemas/usage";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

import { UsageMeter } from "./usage-meter";

interface UsageCardProps {
  snapshot: UsageSnapshot;
}

export function UsageCard({ snapshot }: UsageCardProps) {
  return (
    <Card className="rounded-mochi shadow-sm">
      <CardHeader className="flex flex-row items-center justify-between gap-2">
        <CardTitle className="capitalize">{snapshot.provider}</CardTitle>
        <Badge variant="secondary">{snapshot.source}</Badge>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <UsageMeter label={snapshot.primary.label} usedPercent={snapshot.primary.used_percent} />
        {snapshot.secondary ? (
          <UsageMeter
            label={snapshot.secondary.label}
            usedPercent={snapshot.secondary.used_percent}
          />
        ) : null}
      </CardContent>
    </Card>
  );
}
