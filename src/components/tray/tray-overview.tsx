import type { ProviderId, UsageSnapshot } from "@/lib/schemas/usage";

import { ProviderUsageSection } from "@/components/usage/provider-usage-section";

interface TrayOverviewProps {
  snapshots: UsageSnapshot[];
  onRefreshProvider?: (provider: ProviderId) => void;
  refreshingProvider?: ProviderId | null;
}

export function TrayOverview({
  snapshots,
  onRefreshProvider,
  refreshingProvider = null,
}: TrayOverviewProps) {
  return (
    <div className="flex flex-col gap-4">
      {snapshots.map((snapshot, index) => (
        <ProviderUsageSection
          key={snapshot.provider}
          snapshot={snapshot}
          onRefresh={onRefreshProvider}
          isRefreshing={refreshingProvider === snapshot.provider}
          isLast={index === snapshots.length - 1}
        />
      ))}
    </div>
  );
}
