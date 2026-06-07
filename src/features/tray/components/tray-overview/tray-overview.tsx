import { Fragment } from "react";

import type { ProviderId, ProviderUsageState } from "@/lib/schemas/usage";

import { TrayPanelDivider } from "@/features/tray/components/tray-panel-divider";
import { ProviderUsageSection } from "@/features/usage/components/provider-usage-section";

interface TrayOverviewProps {
  states: ProviderUsageState[];
  onRefreshProvider?: (provider: ProviderId) => void;
  refreshingProvider?: ProviderId | null;
  fillActivationKey?: string;
}

export function TrayOverview({
  states,
  onRefreshProvider,
  refreshingProvider = null,
  fillActivationKey,
}: TrayOverviewProps) {
  return (
    <div className="flex flex-col">
      {states.map((state, index) => (
        <Fragment key={state.provider}>
          <ProviderUsageSection
            state={state}
            snapshot={state.snapshot ?? undefined}
            onRefresh={onRefreshProvider}
            isRefreshing={refreshingProvider === state.provider}
            fillActivationKey={fillActivationKey}
          />
          {index < states.length - 1 ? <TrayPanelDivider /> : null}
        </Fragment>
      ))}
    </div>
  );
}
