import { Link } from "@tanstack/react-router";
import { RefreshCwIcon, SettingsIcon } from "lucide-react";
import { useState } from "react";

import { MochiMascot } from "@/components/mascot/mochi-mascot";
import { TrayOverview } from "@/components/tray/tray-overview";
import { TrayPanelShell } from "@/components/tray/tray-panel-shell";
import { TrayPanelTabList } from "@/components/tray/tray-panel-tab-list";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { UsageCard } from "@/components/usage/usage-card";
import { useRefreshProvider, useSettings } from "@/hooks/use-tray-events";
import { useUsageData } from "@/hooks/use-usage-data";
import {
  buildTrayPanelTabs,
  filterSnapshotsForEnabledProviders,
  getOverviewMetrics,
} from "@/lib/utils/tray-panel-tabs";
import { getMascotStateFromSnapshots } from "@/lib/utils/mascot-state";
import { usageSnapshotsEmptyMessage } from "@/lib/utils/usage-snapshots-empty-message";

export function TrayPanel() {
  const { data: settings } = useSettings();
  const { data, error, isError, isPending, isSuccess, refetch, isFetching } = useUsageData();
  const refreshProvider = useRefreshProvider();
  const [activeTab, setActiveTab] = useState("overview");

  const isRefreshing = isFetching || refreshProvider.isPending;
  const enabledProviders = settings?.enabled_providers ?? [];
  const snapshots = filterSnapshotsForEnabledProviders(data ?? [], enabledProviders);
  const tabs = buildTrayPanelTabs(snapshots, enabledProviders);
  const metrics = getOverviewMetrics(snapshots);
  const mascotState = getMascotStateFromSnapshots(snapshots, { isError });

  return (
    <TrayPanelShell>
      <section className="mx-auto flex w-full max-w-[360px] flex-col">
        <header className="flex items-center justify-between gap-2 px-3 pt-3 pb-2">
          <MochiMascot state={mascotState} className="size-9" />
          <div className="flex items-center gap-0.5">
            <Button
              variant="ghost"
              size="icon-sm"
              className="cursor-pointer"
              disabled={isRefreshing}
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
          data={data}
          error={error}
          isError={isError}
          isPending={isPending}
          isSuccess={isSuccess}
          enabledProviderCount={settings?.enabled_providers.length ?? 0}
          activeTab={activeTab}
          onTabChange={setActiveTab}
          tabs={tabs}
          metrics={metrics}
          snapshots={snapshots}
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
  data: ReturnType<typeof useUsageData>["data"];
  error: ReturnType<typeof useUsageData>["error"];
  isError: boolean;
  isPending: boolean;
  isSuccess: boolean;
  enabledProviderCount: number;
  activeTab: string;
  onTabChange: (value: string) => void;
  tabs: ReturnType<typeof buildTrayPanelTabs>;
  metrics: ReturnType<typeof getOverviewMetrics>;
  snapshots: NonNullable<ReturnType<typeof useUsageData>["data"]>;
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
  metrics,
  snapshots,
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
            <TrayOverview snapshots={snapshots} metrics={metrics} />
          ) : activeSnapshot ? (
            <>
              <UsageCard snapshot={activeSnapshot} compact />
              <p className="text-muted-foreground mt-3 text-[10px]">
                Updated {formatUpdatedAt(activeSnapshot.updated_at)}
              </p>
            </>
          ) : null}
        </div>
      </div>
    );
  }

  return null;
}

function formatUpdatedAt(updatedAt: string): string {
  const date = new Date(updatedAt);
  if (Number.isNaN(date.getTime())) {
    return updatedAt;
  }

  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}
