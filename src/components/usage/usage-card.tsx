import type { ProviderId, UsageSnapshot } from "@/lib/schemas/usage";

import { ProviderUsageSection } from "@/components/usage/provider-usage-section";

interface UsageCardProps {
  snapshot: UsageSnapshot;
  onRefresh?: (provider: ProviderId) => void;
  isRefreshing?: boolean;
}

export function UsageCard({ snapshot, onRefresh, isRefreshing = false }: UsageCardProps) {
  return (
    <ProviderUsageSection
      snapshot={snapshot}
      onRefresh={onRefresh}
      isRefreshing={isRefreshing}
    />
  );
}
