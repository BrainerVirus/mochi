import { useRef } from "react";

import { UsageSnapshotsPanel } from "@/components/tray/tray-panel-content";
import { TrayPanelDivider } from "@/components/tray/tray-panel-divider";
import { TrayPanelFooter } from "@/components/tray/tray-panel-footer";
import { TrayPanelShell } from "@/components/tray/tray-panel-shell";
import { useTrayPanelFocusReset } from "@/hooks/use-tray-panel-focus-reset";
import { useTrayPanelHeight } from "@/hooks/use-tray-panel-height";
import { useTrayPanelShortcuts } from "@/hooks/use-tray-panel-shortcuts";
import { useTrayPanelState } from "@/hooks/use-tray-panel-state";
import { quitApp } from "@/lib/tauri/commands";

export function TrayPanel() {
  const layoutRef = useRef<HTMLDivElement>(null);
  const {
    settings,
    error,
    isError,
    isPending,
    isSuccess,
    isFetching,
    refreshAll,
    isRefreshingAll,
    refreshProviderMutation,
    selectedTab,
    refreshingProvider,
    states,
    tabs,
    handleTabChange,
    handleRefreshProvider,
  } = useTrayPanelState();

  useTrayPanelHeight(layoutRef, selectedTab);
  useTrayPanelFocusReset(layoutRef);
  useTrayPanelShortcuts({
    onRefresh: () => {
      void refreshAll();
    },
    onQuit: () => {
      void quitApp();
    },
  });

  return (
    <TrayPanelShell layoutRef={layoutRef}>
      <section
        data-tray-panel-content
        className="mx-auto flex w-full max-w-[360px] min-w-0 flex-col"
      >
        <UsageSnapshotsPanel
          error={error}
          isError={isError}
          isPending={isPending}
          isSuccess={isSuccess}
          enabledProviderCount={settings?.enabled_providers.length ?? 0}
          activeTab={selectedTab}
          onTabChange={handleTabChange}
          tabs={tabs}
          states={states}
          onRefreshProvider={handleRefreshProvider}
          refreshingProvider={refreshingProvider}
        />
        <TrayPanelDivider inset data-tray-panel-separator />
        <TrayPanelFooter
          isRefreshing={isFetching || refreshProviderMutation.isPending || isRefreshingAll}
          onRefresh={() => {
            void refreshAll();
          }}
          onQuit={() => {
            void quitApp();
          }}
        />
      </section>
    </TrayPanelShell>
  );
}
