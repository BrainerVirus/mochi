import { Link } from "@tanstack/react-router";
import { RefreshCwIcon, SettingsIcon } from "lucide-react";
import { useRef, useState } from "react";

import { useTrayPanelHeight } from "@/hooks/use-tray-panel-height";

import { TrayOverview } from "@/components/tray/tray-overview";
import { TrayPanelShell } from "@/components/tray/tray-panel-shell";
import { TrayPanelTabList } from "@/components/tray/tray-panel-tab-list";
import { ProviderUsageSection } from "@/components/usage/provider-usage-section";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useRefreshProvider, useSettings } from "@/hooks/use-tray-events";
import { useUsageData } from "@/hooks/use-usage-data";
import type { ProviderId } from "@/lib/schemas/usage";
import { buildTrayPanelTabs, filterSnapshotsForTrayPanel } from "@/lib/utils/tray-panel-tabs";
import { usageSnapshotsEmptyMessage } from "@/lib/utils/usage-snapshots-empty-message";

export function TrayPanel() {
  const { data: settings } = useSettings();
  const { data, error, isError, isPending, isSuccess, refetch, isFetching } = useUsageData();
  const refreshProvider = useRefreshProvider();
  const [activeTab, setActiveTab] = useState("overview");
  const [refreshingProvider, setRefreshingProvider] = useState<ProviderId | null>(null);
  const contentRef = useRef<HTMLElement>(null);

  useTrayPanelHeight(contentRef);

  const isRefreshingAll = isFetching || refreshProvider.isPending;
  const enabledProviders = settings?.enabled_providers ?? [];
  const snapshots = filterSnapshotsForTrayPanel(data ?? [], enabledProviders);
  const tabs = buildTrayPanelTabs(data ?? [], enabledProviders);

  function handleRefreshProvider(provider: ProviderId) {
    setRefreshingProvider(provider);
    refreshProvider.mutate(provider, {
      onSettled: () => {
        setRefreshingProvider(null);
      },
    });
  }

  return (
    <TrayPanelShell>
      <section ref={contentRef} className="mx-auto flex w-full max-w-[360px] flex-col">
        <header className="flex items-center justify-between gap-2 px-3 pt-3 pb-2">
          <h1 className="text-sm font-semibold tracking-tight">Usage</h1>
          <div className="flex items-center gap-0.5">
            <Button
              variant="ghost"
              size="icon-sm"
              className="cursor-pointer"
              disabled={isRefreshingAll}
              aria-label="Refresh usage"
              onClick={() => {
                void refetch();
              }}
            >
              <RefreshCwIcon data-icon="inline-start" />
            </Button>
            <Button variant="ghost" size="icon-sm" className="cursor-pointer" asChild>
              <Link to="/settings" aria-label="Open settings" className="cursor-pointer">
                <SettingsIcon data-icon="inline-start" />
              </Link>
            </Button>
          </div>
        </header>

        <UsageSnapshotsPanel
          error={error}
          isError={isError}
          isPending={isPending}
          isSuccess={isSuccess}
          enabledProviderCount={settings?.enabled_providers.length ?? 0}
          activeTab={activeTab}
          onTabChange={setActiveTab}
          tabs={tabs}
          snapshots={snapshots}
          onRefreshProvider={handleRefreshProvider}
          refreshingProvider={refreshingProvider}
        />

        <Separator className="mx-3" />

        <p className="text-muted-foreground px-3 py-2 text-center text-[10px]">
          Left-click tray icon to reopen · Right-click for menu
        </p>
      </section>
    </TrayPanelShell>
  );
}

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

function UsageSnapshotsPanel({
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
            />
          ) : null}
        </div>
      </div>
    );
  }

  return null;
}
