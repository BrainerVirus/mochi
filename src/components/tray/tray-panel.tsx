import { useMemo, useRef, useState } from "react";

import { UsageSnapshotsPanel } from "@/components/tray/tray-panel-content";
import { TrayPanelFooter } from "@/components/tray/tray-panel-footer";
import { TrayPanelShell } from "@/components/tray/tray-panel-shell";
import { useRefreshProvider, useSettings } from "@/hooks/use-tray-events";
import { useTrayPanelHeight } from "@/hooks/use-tray-panel-height";
import { useTrayPanelRefresh } from "@/hooks/use-tray-panel-refresh";
import { useTrayPanelShortcuts } from "@/hooks/use-tray-panel-shortcuts";
import { useUsageData } from "@/hooks/use-usage-data";
import type { ProviderId } from "@/lib/schemas/usage";
import { quitApp } from "@/lib/tauri/commands";
import { buildTrayPanelTabs, filterSnapshotsForTrayPanel } from "@/lib/utils/tray-panel-tabs";

export function TrayPanel() {
  const { data: settings } = useSettings();
  const { data, error, isError, isPending, isSuccess, refetch, isFetching } = useUsageData();
  const refreshProviderMutation = useRefreshProvider();
  const [activeTab, setActiveTab] = useState("overview");
  const [refreshingProvider, setRefreshingProvider] = useState<ProviderId | null>(null);
  const contentRef = useRef<HTMLElement>(null);
  const footerRef = useRef<HTMLElement>(null);

  const enabledProviders = useMemo(
    () => settings?.enabled_providers ?? [],
    [settings?.enabled_providers],
  );
  const { refreshAll, isRefreshingAll } = useTrayPanelRefresh({
    enabledProviders,
    refetch: () => refetch(),
  });

  useTrayPanelHeight({ contentRef, footerRef });

  const snapshots = filterSnapshotsForTrayPanel(data ?? [], enabledProviders);
  const tabs = buildTrayPanelTabs(data ?? [], enabledProviders);
  useTrayPanelShortcuts({
    onRefresh: () => {
      void refreshAll();
    },
    onQuit: () => {
      void quitApp();
    },
  });

  function handleRefreshProvider(provider: ProviderId) {
    setRefreshingProvider(provider);
    refreshProviderMutation.mutate(provider, {
      onSettled: () => {
        setRefreshingProvider(null);
      },
    });
  }

  return (
    <TrayPanelShell
      footer={
        <TrayPanelFooter
          footerRef={footerRef}
          isRefreshing={isFetching || refreshProviderMutation.isPending || isRefreshingAll}
          onRefresh={() => {
            void refreshAll();
          }}
          onQuit={() => {
            void quitApp();
          }}
        />
      }
    >
      <section ref={contentRef} className="mx-auto flex w-full max-w-[360px] flex-col">
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

        <p className="text-muted-foreground px-3 pt-1 pb-2 text-center text-[10px]">
          Left-click tray icon to reopen · Right-click for menu
        </p>
      </section>
    </TrayPanelShell>
  );
}
