import { Fragment } from "react";

import type { ProviderId, UsageSnapshot } from "@/lib/schemas/usage";

import { TrayPanelDivider } from "@/components/tray/tray-panel-divider";
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
    <div className="flex flex-col">
      {snapshots.map((snapshot, index) => (
        <Fragment key={snapshot.provider}>
          <div data-tray-tab-enter className="flex flex-col">
            <ProviderUsageSection
              snapshot={snapshot}
              onRefresh={onRefreshProvider}
              isRefreshing={refreshingProvider === snapshot.provider}
            />
            {index < snapshots.length - 1 ? <TrayPanelDivider /> : null}
          </div>
        </Fragment>
      ))}
    </div>
  );
}
