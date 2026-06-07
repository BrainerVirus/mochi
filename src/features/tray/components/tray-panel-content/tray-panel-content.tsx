import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { ProviderUsageSection } from "@/components/usage/provider-usage-section";
import { TrayOverview } from "@/features/tray/components/tray-overview";
import { TrayPanelTabList } from "@/features/tray/components/tray-panel-tab-list";
import { useTabFillActivationKey } from "@/features/tray/hooks/use-tab-fill-activation-key/use-tab-fill-activation-key";
import { useUsageData } from "@/hooks/use-usage-data";
import type { ProviderId } from "@/lib/schemas/usage";
import { trayPanelSpacing } from "@/lib/utils/tray-panel-spacing";
import { buildTrayPanelTabsFromStates } from "@/lib/utils/tray-panel-tabs";
import { usageSnapshotsEmptyMessage } from "@/lib/utils/usage-snapshots-empty-message";

interface UsageSnapshotsPanelProps {
  error: ReturnType<typeof useUsageData>["error"];
  isPending: boolean;
  isError: boolean;
  isSuccess: boolean;
  enabledProviderCount: number;
  activeTab: string;
  onTabChange: (value: string) => void;
  tabs: ReturnType<typeof buildTrayPanelTabsFromStates>;
  states: NonNullable<ReturnType<typeof useUsageData>["data"]>;
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
  states,
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

  if (isSuccess && states.length === 0) {
    return (
      <p className="text-muted-foreground px-3 py-6 text-center text-xs">
        {usageSnapshotsEmptyMessage(enabledProviderCount)}
      </p>
    );
  }

  if (isSuccess && states.length > 0) {
    const activeState = states.find((state) => state.provider === activeTab);

    return (
      <TabUsageContent
        activeTab={activeTab}
        activeState={activeState}
        states={states}
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
  activeState,
  states,
  tabs,
  onTabChange,
  onRefreshProvider,
  refreshingProvider,
}: {
  activeTab: string;
  activeState: UsageSnapshotsPanelProps["states"][number] | undefined;
  states: UsageSnapshotsPanelProps["states"];
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
            states={states}
            onRefreshProvider={onRefreshProvider}
            refreshingProvider={refreshingProvider}
            fillActivationKey={fillActivationKey}
          />
        ) : activeState ? (
          <ProviderUsageSection
            key={fillActivationKey}
            state={activeState}
            snapshot={activeState.snapshot ?? undefined}
            onRefresh={onRefreshProvider}
            isRefreshing={refreshingProvider === activeState.provider}
            showProviderActions
            fillActivationKey={fillActivationKey}
          />
        ) : null}
      </div>
    </div>
  );
}
