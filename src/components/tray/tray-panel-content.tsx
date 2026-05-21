import { TrayOverview } from "@/components/tray/tray-overview";
import { TrayPanelTabList } from "@/components/tray/tray-panel-tab-list";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ProviderUsageSection } from "@/components/usage/provider-usage-section";
import { useUsageData } from "@/hooks/use-usage-data";
import type { ProviderId } from "@/lib/schemas/usage";
import { buildTrayPanelTabs } from "@/lib/utils/tray-panel-tabs";
import { usageSnapshotsEmptyMessage } from "@/lib/utils/usage-snapshots-empty-message";

interface UsageSnapshotsPanelProps {
  error: ReturnType<typeof useUsageData>["error"];
  isError: boolean;
  isPending: boolean;
  isSuccess: boolean;
  enabledProviderCount: number;
  activeTab: string;
  onTabChange: (value: string) => void;
  tabs: ReturnType<typeof buildTrayPanelTabs>;
  snapshots: NonNullable<ReturnType<typeof useUsageData>["data"]>;
  onRefreshProvider: (provider: ProviderId) => void;
  refreshingProvider: ProviderId | null;
}

export function UsageSnapshotsPanel({
  error,
  isError,
  isPending,
  isSuccess,
  enabledProviderCount,
  activeTab,
  onTabChange,
  tabs,
  snapshots,
  onRefreshProvider,
  refreshingProvider,
}: UsageSnapshotsPanelProps) {
  if (isPending) {
    return (
      <output className="text-muted-foreground block px-3 py-6 text-center text-xs">
        Loading provider usage…
      </output>
    );
  }

  if (isError) {
    return (
      <div className="px-3 pb-3">
        <Alert variant="destructive">
          <AlertTitle>Could not load usage</AlertTitle>
          <AlertDescription>{error?.message ?? "Unknown error"}</AlertDescription>
        </Alert>
      </div>
    );
  }

  if (isSuccess && snapshots.length === 0) {
    return (
      <p className="text-muted-foreground px-3 py-6 text-center text-xs">
        {usageSnapshotsEmptyMessage(enabledProviderCount)}
      </p>
    );
  }

  if (isSuccess && snapshots.length > 0) {
    const activeSnapshot = snapshots.find((snapshot) => snapshot.provider === activeTab);

    return (
      <div className="flex flex-col gap-0">
        <TrayPanelTabList tabs={tabs} value={activeTab} onValueChange={onTabChange} />

        <div className="px-3 py-3">
          {activeTab === "overview" ? (
            <TrayOverview
              snapshots={snapshots}
              onRefreshProvider={onRefreshProvider}
              refreshingProvider={refreshingProvider}
            />
          ) : activeSnapshot ? (
            <ProviderUsageSection
              snapshot={activeSnapshot}
              onRefresh={onRefreshProvider}
              isRefreshing={refreshingProvider === activeSnapshot.provider}
              showProviderActions
            />
          ) : null}
        </div>
      </div>
    );
  }

  return null;
}
