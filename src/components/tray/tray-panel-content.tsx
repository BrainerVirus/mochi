import { TrayOverview } from "@/components/tray/tray-overview";
import { TrayPanelTabList } from "@/components/tray/tray-panel-tab-list";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ProviderUsageSection } from "@/components/usage/provider-usage-section";
import { useTabFillActivationKey } from "@/hooks/use-tab-fill-activation-key";
import { useUsageData } from "@/hooks/use-usage-data";
import type { ProviderId } from "@/lib/schemas/usage";
import { trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";
import { buildTrayPanelTabs } from "@/lib/utils/tray-panel-tabs";
import { usageSnapshotsEmptyMessage } from "@/lib/utils/usage-snapshots-empty-message";

interface UsageSnapshotsPanelProps {
  error: ReturnType<typeof useUsageData>["error"];
  isPending: boolean;
  isError: boolean;
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
      <TabUsageContent
        activeTab={activeTab}
        activeSnapshot={activeSnapshot}
        snapshots={snapshots}
        tabs={tabs}
        onTabChange={onTabChange}
        onRefreshProvider={onRefreshProvider}
        refreshingProvider={refreshingProvider}
      />
    );
  }

  return null;
}

function TabUsageContent({
  activeTab,
  activeSnapshot,
  snapshots,
  tabs,
  onTabChange,
  onRefreshProvider,
  refreshingProvider,
}: {
  activeTab: string;
  activeSnapshot: UsageSnapshotsPanelProps["snapshots"][number] | undefined;
  snapshots: UsageSnapshotsPanelProps["snapshots"];
  tabs: UsageSnapshotsPanelProps["tabs"];
  onTabChange: UsageSnapshotsPanelProps["onTabChange"];
  onRefreshProvider: UsageSnapshotsPanelProps["onRefreshProvider"];
  refreshingProvider: UsageSnapshotsPanelProps["refreshingProvider"];
}) {
  const fillActivationKey = useTabFillActivationKey(activeTab);

  return (
    <div className="flex flex-col gap-0">
      <TrayPanelTabList tabs={tabs} value={activeTab} onValueChange={onTabChange} />

      <div className={`${trayPanelSpacing.contentX} ${trayPanelSpacing.contentTop} pb-0`}>
        {activeTab === "overview" ? (
          <TrayOverview
            key={fillActivationKey}
            snapshots={snapshots}
            onRefreshProvider={onRefreshProvider}
            refreshingProvider={refreshingProvider}
            fillActivationKey={fillActivationKey}
          />
        ) : activeSnapshot ? (
          <ProviderUsageSection
            key={fillActivationKey}
            snapshot={activeSnapshot}
            onRefresh={onRefreshProvider}
            isRefreshing={refreshingProvider === activeSnapshot.provider}
            showProviderActions
            fillActivationKey={fillActivationKey}
          />
        ) : null}
      </div>
    </div>
  );
}
