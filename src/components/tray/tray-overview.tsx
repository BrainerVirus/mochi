import { Fragment } from "react";

import type { ProviderId, UsageSnapshot } from "@/lib/schemas/usage";

import { TrayPanelDivider } from "@/components/tray/tray-panel-divider";
import { ProviderUsageSection } from "@/components/usage/provider-usage-section";

interface TrayOverviewProps {
  snapshots: UsageSnapshot[];
  onRefreshProvider?: (provider: ProviderId) => void;
  refreshingProvider?: ProviderId | null;
  fillActivationKey?: string;
}

export function TrayOverview({
  snapshots,
  onRefreshProvider,
  refreshingProvider = null,
  fillActivationKey,
}: TrayOverviewProps) {
  return (
    <div className="flex flex-col">
      {snapshots.map((snapshot, index) => (
        <Fragment key={snapshot.provider}>
          <ProviderUsageSection
            snapshot={snapshot}
            onRefresh={onRefreshProvider}
            isRefreshing={refreshingProvider === snapshot.provider}
            fillActivationKey={fillActivationKey}
          />
          {index < snapshots.length - 1 ? <TrayPanelDivider /> : null}
        </Fragment>
      ))}
    </div>
  );
}
